
use std::any::Any;
use std::cell::RefCell;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::Debug;

/* this is done by somewhat following the python data model in https://docs.python.org/3/reference/datamodel.html */

pub type MemoryAddress = usize;

pub const BUILTIN_MODULE: &'static str = "__builtin__";

#[derive(Debug, Clone)]
pub enum PyObjectStructure {
    None,
    NotImplemented,
    Object {
        raw_data: MemoryAddress,
    },
    Callable {
        code: MemoryAddress,
        name: Option<String>
    },
    Type {
        name: String,
        bounded_functions: HashMap<String, MemoryAddress>,
        unbounded_functions: HashMap<String, MemoryAddress>,
    },
    Module {
        name: String,
        objects: HashMap<String, MemoryAddress>
    }
}

#[derive(Debug, Clone)]
pub struct PyObject {
    pub type_addr: MemoryAddress,
    pub structure: PyObjectStructure,
}

pub struct CallParams {
    pub bound_pyobj: Option<MemoryAddress>,
    pub func_address: MemoryAddress,
    pub func_name: Option<String>,
    pub params: Vec<MemoryAddress>,
}

pub struct PyCallable {
    pub code: Box<dyn Fn(&Interpreter, CallParams) -> MemoryAddress>,
}

pub struct MemoryCell {
    pub data: UnsafeCell<Box<dyn Any>>,
    pub valid: bool,
}

pub struct Memory {
    pub memory: RefCell<Vec<MemoryCell>>,
    pub recently_deallocated_indexes: RefCell<Vec<MemoryAddress>>,
}

pub struct UninitializedMemory {}

impl Memory {
    pub fn get<'a>(&'a self, address: MemoryAddress) -> &mut Box<dyn Any> {
        {
            if address > self.memory.borrow().len() - 1 {
                panic!(
                    "Attempt to read from uninitialized memory at address {}",
                    address
                );
            }
        }
        let cell = &self.memory.borrow()[address];
        if cell.valid {
            //SAFETY: Python doesn't have ownership and borrowing, and we're trying to run a python program evaluated in Rust.
            return unsafe { &mut *cell.data.get() };
        } else {
            panic!("Attempt to read from non-valid memory address {}", address);
        }
    }

    pub fn write<'a>(&'a self, address: MemoryAddress, data: Box<dyn Any>) {
        {
            if address > self.memory.borrow().len() - 1 {
                panic!(
                    "Attempt to write in uninitialized memory at address {}",
                    address
                );
            }
        }

        let cell = &mut self.memory.borrow_mut()[address];
        if cell.valid {
            cell.data = UnsafeCell::new(data);
        } else {
            panic!("Attempt to write in non-valid memory address {}", address);
        }
    }

    #[cfg(test)]
    pub fn deallocate<'a>(&'a self, address: MemoryAddress) {
        {
            if address > self.memory.borrow().len() - 1 {
                panic!(
                    "Attempt to deallocate non-existent memory at address {}",
                    address
                );
            }
        }
        {
            let cell = &mut self.memory.borrow_mut()[address];
            if cell.valid {
                cell.data = UnsafeCell::new(Box::new(UninitializedMemory {}));
                cell.valid = false;
            } else {
                panic!("Attempt to dealloate already invalid memory at address in non-valid memory address {}", address)
            }
        }
        self.recently_deallocated_indexes.borrow_mut().push(address);
    }

    pub fn allocate<'a>(&'a self) -> MemoryAddress {
        let dealloc: Option<MemoryAddress>;

        {
            dealloc = self.recently_deallocated_indexes.borrow_mut().pop();
        }

        match dealloc {
            Some(address) => {
                let mut cell = &mut self.memory.borrow_mut()[address];
                if cell.valid {
                    panic!(
                        "Attempt to allocate onto already occupied address {}",
                        address
                    )
                } else {
                    cell.data = UnsafeCell::new(Box::new(UninitializedMemory {}));
                    cell.valid = true;
                }
                return address;
            }
            None => {
                let mut borrow = self.memory.borrow_mut();
                borrow.push(MemoryCell {
                    data: UnsafeCell::new(Box::new(UninitializedMemory {})),
                    valid: true,
                });
                return borrow.len() - 1;
            }
        };
    }

    pub fn allocate_and_write<'a>(&'a self, data: Box<dyn Any>) -> MemoryAddress {
        let address = self.allocate();
        self.write(address, data);
        return address;
    }
}

pub struct StackFrame {
    pub values: HashMap<String, MemoryAddress>,
    pub stack: Vec<MemoryAddress>,
    pub current_function: MemoryAddress
}

pub struct InterpreterSpecialValues {
    pub type_type: MemoryAddress,
    pub none_type: MemoryAddress,
    pub none_value: MemoryAddress,
    pub not_implemented_type: MemoryAddress,
    pub not_implemented_value: MemoryAddress,
    pub callable_type: MemoryAddress,
    pub module_type: MemoryAddress
}

pub struct Interpreter {
    pub stack: RefCell<Vec<StackFrame>>,
    pub memory: Memory,
    pub special_values: InterpreterSpecialValues,
    pub modules: RefCell<HashMap<String, MemoryAddress>>,
    pub prog_counter: Cell<usize>
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut interpreter = Interpreter {
            stack: RefCell::new(vec![StackFrame{
                values: HashMap::new(),
                current_function: 0,
                stack: vec![]
            }]),
            memory: Memory {
                memory: RefCell::new(vec![]),
                recently_deallocated_indexes: RefCell::new(vec![]),
            },
            special_values: InterpreterSpecialValues {
                type_type: 0,
                none_type: 0,
                none_value: 0,
                callable_type: 0,
                not_implemented_type: 0,
                not_implemented_value: 0,
                module_type: 0
            },
            modules: RefCell::new(HashMap::new()),
            prog_counter: Cell::new(0)
        };

        let type_type = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: 0,
            structure: PyObjectStructure::Type {
                name: String::from("type"),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new()
            },
        }));

        let none_type = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: type_type,
            structure: PyObjectStructure::Type {
                name: String::from("NoneType"),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new()
            },
        }));

        let none_value = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: none_type,
            structure: PyObjectStructure::None,
        }));

        let not_implemented_type = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: type_type,
            structure: PyObjectStructure::Type {
                name: String::from("NotImplemented"),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new()
            },
        }));

        let not_implemented_value = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: not_implemented_type,
            structure: PyObjectStructure::NotImplemented,
        }));

        let callable_type = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: type_type,
            structure: PyObjectStructure::Type {
                name: String::from("function"),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new()
            },
        }));

        let module_type = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: type_type,
            structure: PyObjectStructure::Type {
                name: String::from("module"),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new()
            },
        }));

        let builtin_module_obj = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: module_type,
            structure: PyObjectStructure::Module {
                name: BUILTIN_MODULE.to_string(),
                objects: HashMap::new()
            }
        }));

        interpreter.special_values.type_type = type_type;
        interpreter.special_values.none_type = none_type;
        interpreter.special_values.none_value = none_value;
        interpreter.special_values.not_implemented_type = not_implemented_type;
        interpreter.special_values.not_implemented_value = not_implemented_value;
        interpreter.special_values.callable_type = callable_type;
        interpreter.special_values.module_type = module_type;
        interpreter.modules.borrow_mut().insert(BUILTIN_MODULE.to_string(), builtin_module_obj);

        return interpreter;
    }

    pub fn create_type(
        &self,
        module: &str,
        name: &str,
        methods: HashMap<String, MemoryAddress>,
        functions: HashMap<String, MemoryAddress>
    ) -> MemoryAddress {
        let created_type = PyObject {
            type_addr: self.special_values.type_type,
            structure: PyObjectStructure::Type {
                name: name.to_string(),
                bounded_functions: methods,
                unbounded_functions: functions
            },
        };
        let type_address = self.allocate_and_write(Box::new(created_type));
        match self.modules.borrow().get(module) {
            Some(module_addr) => {
                match self.get_pyobj_byaddr(*module_addr) {
                    Some(pyobj) => {
                        match &mut pyobj.structure {
                            PyObjectStructure::Module{name: _, objects} => {

                                match objects.get(name) {
                                    Some(_) => {
                                        panic!("Name already exists in module {}: {}", module, name);
                                    },
                                    None => {
                                        objects.insert(name.to_string(), type_address);
                                        return type_address;
                                    }
                                }
                            },
                            _ => {
                                panic!("Module name {} was found but it's not actually a module: {:?}", module, pyobj.structure);
                            }
                        }
                    },
                    None => {
                        panic!("Module name {} was found but does not exist in memory: {}", module, module_addr);
                    }
                }
            },
            None => {
                panic!("Module does not exist: {}", module);
            }
        }
    }

    pub fn add_to_module(
        &self,
        module: &str,
        name: &str,
        pyobject_addr: MemoryAddress
    ) {
        match self.modules.borrow().get(module) {
            Some(module_addr) => {
                match self.get_pyobj_byaddr(*module_addr) {
                    Some(pyobj) => {
                        match &mut pyobj.structure {
                            PyObjectStructure::Module{name: _, objects} => {

                                match objects.get(name) {
                                    Some(_) => {
                                        panic!("Name already exists in module {}: {}", module, name);
                                    },
                                    None => {
                                        objects.insert(name.to_string(), pyobject_addr);
                                    }
                                }
                            },
                            _ => {
                                panic!("Module name {} was found but it's not actually a module: {:?}", module, pyobj.structure);
                            }
                        }
                    },
                    None => {
                        panic!("Module name {} was found but does not exist in memory: {}", module, module_addr);
                    }
                }
            },
            None => {
                panic!("Module does not exist: {}", module);
            }
        }
    }

    pub fn find_module(&self, module: &str) ->Option<&mut PyObject> {
        return self.modules.borrow().get(module).and_then(|v| 
            self.get_pyobj_byaddr(*v)
        );
    }

    pub fn find_in_module(&self, module: &str, name: &str) -> Option<MemoryAddress> {
        return self.find_module(module).and_then(|obj|
            match &obj.structure {
                PyObjectStructure::Module{name: _, objects} => objects.get(name).map(|addr| *addr), 
                _ => panic!("Object is not module: {:?}", obj.structure)
            }
        )
    }

    pub fn get_pyobj_byaddr(&self, addr: MemoryAddress) -> Option<&mut PyObject> {
        return self.memory.get(addr).downcast_mut::<PyObject>();
    }

    pub fn get_rawdata_byaddr(&self, addr: MemoryAddress) -> &mut Box<dyn Any> {
        return self.memory.get(addr);
    }

    pub fn get_pyobj_type_byname(&self, module: &str, name: &str) -> Option<(&mut PyObject, MemoryAddress)> {
        return self.find_in_module(module, name).and_then(|addr: MemoryAddress| {
            self.get_pyobj_byaddr(addr).map(|pyobj| (pyobj, addr))
        });
    }
    
    pub fn get_type_method_addr_byname(&self, type_addr: MemoryAddress, method_name: &str) -> Option<MemoryAddress> {
        self.get_pyobj_byaddr(type_addr).and_then(|obj| -> Option<MemoryAddress> {
            let result = match &obj.structure {
                PyObjectStructure::Type {
                    name: _,
                    bounded_functions,
                    unbounded_functions: _
                } => {
                    match bounded_functions.get(method_name) {
                        Some(addr) => Some(*addr),
                        None => None
                    }
                },
                _ => {
                    None
                }
            };
            return result;
        })
    }


    pub fn get_type_name(&self, addr: MemoryAddress) -> &str {
        match self.get_pyobj_byaddr(addr) {
            Some(obj) => match &obj.structure {
                PyObjectStructure::Type {
                    name,
                    bounded_functions: _,
                    unbounded_functions: _
                } => {
                    return &name;
                }
                _ => {
                    panic!(
                        "Attempt to get type name on a non-type object {:?}",
                        obj.structure
                    );
                }
            },
            None => {
                panic!(
                    "Attempt to get type name on non-existing type object for address {}",
                    addr
                );
            }
        }
    }

    pub fn get_pyobj_type_name(&self, addr: MemoryAddress) -> &str {
        match self.get_pyobj_byaddr(addr) {
            Some(obj) => {
                return self.get_type_name(obj.type_addr);
            }
            None => {
                panic!("Attempt to get type name on non-existing object");
            }
        }
    }

    pub fn allocate_module_type_byname_raw(&self, module: &str, name: &str, raw_data: Box<dyn Any>) -> MemoryAddress {
        match self.get_pyobj_type_byname(module, name) {
            Some((_, addr)) => {
                return self.allocate_type_byaddr_raw(addr, raw_data);
            }
            None => {
                panic!("Type {} not found", name);
            }
        }
    }

    pub fn allocate_type_byname_raw(&self, name: &str, raw_data: Box<dyn Any>) -> MemoryAddress {
        self.allocate_module_type_byname_raw(BUILTIN_MODULE, name, raw_data)
    }

    pub fn allocate_type_byaddr_raw_struct(
        &self,
        type_addr: MemoryAddress,
        structure: PyObjectStructure,
    ) -> MemoryAddress {
        let obj = PyObject {
            type_addr,
            structure,
        };
        return self.allocate_and_write(Box::new(obj));
    }

    pub fn allocate_type_byaddr_raw(
        &self,
        type_addr: MemoryAddress,
        raw_data: Box<dyn Any>,
    ) -> MemoryAddress {
        let raw_data_ptr = self.allocate_and_write(raw_data);

        return self.allocate_type_byaddr_raw_struct(
            type_addr,
            PyObjectStructure::Object {
                raw_data: raw_data_ptr,
            },
        );
    }

    #[cfg(test)]
    pub fn allocate_type_byname_raw_struct(
        &self,
        module: &str,
        name: &str,
        raw_data: Box<dyn Any>,
    ) -> MemoryAddress {
        let (_, type_addr) = self.get_pyobj_type_byname(module, name).unwrap();
        self.allocate_type_byaddr_raw(type_addr, raw_data)
    }

    pub fn create_callable_pyobj(&self, callable: PyCallable, name: Option<String>) -> MemoryAddress {
        let raw_callable = self.allocate_and_write(Box::new(callable));

        self.allocate_type_byaddr_raw_struct(
            self.special_values.callable_type,
            PyObjectStructure::Callable { code: raw_callable, name },
        )
    }

    pub fn get_raw_data_of_pyobj<T>(&self, addr: MemoryAddress) -> &T
    where
        T: 'static,
    {
        self.get_raw_data_of_pyobj_opt(addr).unwrap()
    }

    pub fn get_raw_data_of_pyobj_opt<T>(&self, addr: MemoryAddress) -> Option<&T>
    where
        T: 'static,
    {
        let pyobj = self.get_pyobj_byaddr(addr).unwrap();
        if let PyObjectStructure::Object {
            raw_data,
        } = &pyobj.structure
        {
            let rawdata = self.get_rawdata_byaddr(*raw_data);
            return rawdata.downcast_ref::<T>();
        } else {
            panic!(
                "get_raw_data_of_pyobj cannot be called on {:?}",
                pyobj.structure
            )
        }
    }

    pub fn callable_call(
        &self,
        callable_addr: MemoryAddress,
        call_params: CallParams,
    ) -> MemoryAddress {
        let callable_pyobj = self.get_pyobj_byaddr(callable_addr).unwrap();

        if let PyObjectStructure::Callable { code, name: _ } = &callable_pyobj.structure {
            let pyobj_callable = self
                .get_rawdata_byaddr(*code)
                .downcast_ref::<PyCallable>()
                .unwrap();
            return (pyobj_callable.code)(self, call_params);
        } else {
            panic!("Object is not callable: {:?}", callable_pyobj);
        }
    }

    #[cfg(test)]
    pub fn bounded_function_call(
        &self,
        bound_obj_addr: MemoryAddress,
        method_name: &str,
        params: Vec<MemoryAddress>,
    ) -> MemoryAddress {
        let pyobj: &PyObject = self.get_pyobj_byaddr(bound_obj_addr).unwrap();
        let type_addr = pyobj.type_addr;
        let type_pyobj = self.get_pyobj_byaddr(type_addr).unwrap();

        //type_pyobj must be a type
        if let PyObjectStructure::Type {
            name: _,
            bounded_functions,
            unbounded_functions: _,
        } = &type_pyobj.structure
        {
            let memory_addr = bounded_functions.get(method_name);
            if let Some(method_addr) = memory_addr {
                let call_params = CallParams {
                    bound_pyobj: Some(bound_obj_addr),
                    func_address: *method_addr,
                    func_name: Some(method_name.to_string()),
                    params: params,
                };

                return self.callable_call(*method_addr, call_params);
            } else {
                panic!("Method not found: {}", method_name);
            }
        } else {
            panic!(
                "FATAL ERROR: pyobj addr {} was supposed to be a type, but it's something else: {:?}",
                type_addr, &type_pyobj
            );
        }
    }

    pub fn bounded_function_call_byaddr(
        &self,
        bound_obj_addr: MemoryAddress,
        method_addr: MemoryAddress,
        params: Vec<MemoryAddress>,
    ) -> MemoryAddress {
        let pyobj: &PyObject = self.get_pyobj_byaddr(bound_obj_addr).unwrap();
        let type_addr = pyobj.type_addr;
        let type_pyobj = self.get_pyobj_byaddr(type_addr).unwrap();

        //type_pyobj must be a type
        if let PyObjectStructure::Type {
            name: _,
            bounded_functions: _,
            unbounded_functions: _,
        } = &type_pyobj.structure
        {
            let bounded_method = self.get_pyobj_byaddr(method_addr);
            if let Some(bounded_method_pyobj) = bounded_method {

                match &bounded_method_pyobj.structure {
                    PyObjectStructure::Callable{code: _, name} => {
                        let call_params = CallParams {
                            bound_pyobj: Some(bound_obj_addr),
                            func_address: method_addr,
                            func_name: match name {
                                Some(fname) => Some(fname.to_string()),
                                None => None
                            },
                            params: params,
                        };
        
                        return self.callable_call(method_addr, call_params);
                    },
                    _ => {
                        panic!("Not a method at addr: {}", method_addr);
                    }
                };
                
            } else {
                panic!("Method not found at addr: {}", method_addr);
            }
        } else {
            panic!(
                "FATAL ERROR: pyobj addr {} was supposed to be a type, but it's something else: {:?}",
                type_addr, &type_pyobj
            );
        }
    }

    pub fn unbounded_function_call_byaddr(
        &self,
        function_addr: MemoryAddress,
        params: Vec<MemoryAddress>,
    ) -> MemoryAddress {
        let unbounded_func = self.get_pyobj_byaddr(function_addr);
        if let Some(unbounded_func_pyobj) = unbounded_func {
            return match &unbounded_func_pyobj.structure {
                PyObjectStructure::Callable{code: _, name} => {
                    let call_params = CallParams {
                        bound_pyobj: None,
                        func_address: function_addr,
                        func_name: match name {
                            Some(fname) => Some(fname.to_string()),
                            None => None
                        },
                        params: params,
                    };

                    self.callable_call(function_addr, call_params)
                },
                _ => {
                    panic!("Not a function at addr: {}", function_addr);
                }
            }
        } else {
            panic!("Function not found at addr: {}", function_addr);
        }
    }


    pub fn new_stack_frame(&self) {
        let mut current_stack = self.stack.borrow_mut();
        current_stack.push(StackFrame {
            values: HashMap::new(),
            stack: vec![],
            current_function: 0
        })
    }

    #[cfg(test)]
    pub fn stack_size(&self) -> usize {
        self.stack.borrow().last().unwrap().stack.len()
    }

    pub fn pop_stack(&self) -> MemoryAddress {
        match self.stack.borrow_mut().last_mut().unwrap().stack.pop() {
            Some(addr) => addr,
            None => panic!("Attempt to pop on empty stack!")
        }
    }

    pub fn push_stack(&self, value: MemoryAddress) {
        self.stack.borrow_mut().last_mut().unwrap().stack.push(value)
    }

    pub fn pop_stack_frame(&self) -> StackFrame {
        let mut current_stack = self.stack.borrow_mut();
        match current_stack.pop() {
            Some(stack_frame) => stack_frame,
            None => panic!("Attempt to pop on empty stack frames!")
        }
    }

    #[cfg(test)]
    pub fn bind_local(&self, name: &str, addr: MemoryAddress) {
        let mut current_stack = self.stack.borrow_mut();
        let current_frame = current_stack.last_mut().unwrap();
        current_frame.values.insert(name.to_string(), addr);
    }

    #[cfg(test)]
    pub fn get_local(&self, name: &str) -> MemoryAddress {
        let mut current_stack = self.stack.borrow_mut();
        let current_frame = current_stack.last_mut().unwrap();
        *current_frame.values.get(name).unwrap()
    }

    #[cfg(test)]
    pub fn unbind_local(&self, name: &str) -> MemoryAddress {
        let mut current_stack = self.stack.borrow_mut();
        let current_frame = current_stack.last_mut().unwrap();
        current_frame.values.remove(name).unwrap()
    }

    pub fn allocate_and_write<'a>(&'a self, data: Box<dyn Any>) -> MemoryAddress {
        self.memory.allocate_and_write(data)
    }

    pub fn get_pc(&self) -> usize {
        self.prog_counter.get()
    }

    pub fn jump_pc(&self, delta: usize) -> usize {
        self.prog_counter.set(self.prog_counter.get() + delta);
        self.prog_counter.get()
    }
}

pub fn check_builtin_func_params(name: &str, expected: usize, received: usize) {
    if expected != received {
        panic!(
            "{}() expected {} arguments, got {}",
            name, expected, received
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_types::{register_builtins};

    #[test]
    fn simply_instantiate_int() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let pyobj_int_addr = interpreter.allocate_type_byname_raw("int", Box::new(1 as i128));
        let result_value = interpreter.get_raw_data_of_pyobj::<i128>(pyobj_int_addr);
        assert_eq!(1, *result_value);
    }

    #[test]
    fn call_int_add_int() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let number1 = interpreter.allocate_type_byname_raw("int", Box::new(1 as i128));
        let number2 = interpreter.allocate_type_byname_raw("int", Box::new(3 as i128));

        //number1.__add__(number2)
        let result = interpreter.bounded_function_call(number1, "__add__", vec![number2]);
        let result_value = *interpreter.get_raw_data_of_pyobj::<i128>(result);

        assert_eq!(result_value, 4);
    }

    #[test]
    fn call_int_add_float() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let number1 = interpreter.allocate_type_byname_raw("int", Box::new(1 as i128));
        let number2 = interpreter.allocate_type_byname_raw("float", Box::new(3.5 as f64));

        //number1.__add__(number2)
        let result = interpreter.bounded_function_call(number1, "__add__", vec![number2]);
        let result_value = *interpreter.get_raw_data_of_pyobj::<f64>(result);

        assert_eq!(result_value, 4.5 as f64);
    }

    #[test]
    fn call_float_add_int() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let number1 = interpreter.allocate_type_byname_raw("float", Box::new(3.5 as f64));
        let number2 = interpreter.allocate_type_byname_raw("int", Box::new(1 as i128));

        //number1.__add__(number2)
        let result = interpreter.bounded_function_call(number1, "__add__", vec![number2]);
        let result_value = *interpreter.get_raw_data_of_pyobj::<f64>(result);

        assert_eq!(result_value, 4.5 as f64);
    }


    #[test]
    fn call_float_add_float() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let number1 = interpreter.allocate_type_byname_raw("float", Box::new(3.4 as f64));
        let number2 = interpreter.allocate_type_byname_raw("float", Box::new(1.1 as f64));

        //number1.__add__(number2)
        let result = interpreter.bounded_function_call(number1, "__add__", vec![number2]);
        let result_value = *interpreter.get_raw_data_of_pyobj::<f64>(result);

        assert_eq!(result_value, 4.5 as f64);
    }

    #[test]
    fn call_float_mul_float() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let number1 = interpreter.allocate_type_byname_raw("float", Box::new(2.0 as f64));
        let number2 = interpreter.allocate_type_byname_raw("float", Box::new(3.0 as f64));

        //number1.__add__(number2)
        let result = interpreter.bounded_function_call(number1, "__mul__", vec![number2]);
        let result_value = *interpreter.get_raw_data_of_pyobj::<f64>(result);

        assert_eq!(result_value, 6.0 as f64);
    }

    #[test]
    fn call_int_mul_int() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let number1 = interpreter.allocate_type_byname_raw("int", Box::new(2 as i128));
        let number2 = interpreter.allocate_type_byname_raw("int", Box::new(3 as i128));

        //number1.__add__(number2)
        let result = interpreter.bounded_function_call(number1, "__mul__", vec![number2]);
        let result_value = *interpreter.get_raw_data_of_pyobj::<i128>(result);

        assert_eq!(result_value, 6 as i128);
    }

    #[test]
    fn bind_local_test() {
        let interpreter = Interpreter::new();
        register_builtins(&interpreter);
        let number = interpreter.allocate_type_byname_raw("int", Box::new(17 as i128));
        interpreter.bind_local("x", number);

        let addr_local = interpreter.get_local("x");
        let result_value = *interpreter.get_raw_data_of_pyobj::<i128>(addr_local);

        assert_eq!(result_value, 17 as i128);
    }

 
}
