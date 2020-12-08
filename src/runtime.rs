
use std::any::Any;
use std::cell::RefCell;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::Debug;

/* this is done by somewhat following the python data model in https://docs.python.org/3/reference/datamodel.html */

pub type MemoryAddress = usize;

pub const BUILTIN_MODULE: &'static str = "__builtin__";

pub enum PyObjectStructure {
    None,
    NotImplemented,
    Object {
        raw_data: Box<dyn Any>,
        refcount: usize
    },
    Callable {
        code: Box<dyn Fn(&Runtime, CallParams) -> MemoryAddress>,
        name: Option<String>
    },
    Type {
        name: String,
        bounded_functions: HashMap<String, MemoryAddress>,
        unbounded_functions: HashMap<String, MemoryAddress>,
        supertype: Option<MemoryAddress>
    },
    Module {
        name: String,
        objects: HashMap<String, MemoryAddress>
    }
}

pub struct PyObject {
    pub type_addr: MemoryAddress,
    pub structure: PyObjectStructure,
}

pub struct CallParams<'a> {
    pub bound_pyobj: Option<MemoryAddress>,
    pub func_address: MemoryAddress,
    pub func_name: Option<&'a str>,
    pub params: &'a [MemoryAddress],
}

pub struct PyCallable {
    pub code: Box<dyn Fn(&Runtime, CallParams) -> MemoryAddress>,
}

pub struct Null {}

pub struct MemoryCell {
    pub data: PyObject,
    pub valid: bool,
    pub is_const: bool,
}

pub struct Memory {
    pub memory: Vec<MemoryCell>,
    //gc graph stores: Key = memory addresses, Values = Other adresses that point to that memory addr
    //Every memory address has an entry here
    //pub gc_graph: HashMap<MemoryAddress, Vec<MemoryAddress>>,
    pub recently_deallocated_indexes: Vec<MemoryAddress>,
}

pub struct MemoryStatistics {
    pub allocated_slots: usize,
    pub slots_in_use: usize
}

impl Memory {
    pub fn get(&self, address: MemoryAddress) -> &PyObject {
        let cell = &self.memory[address];
        if cell.valid {
            return &cell.data;
        } else {
            panic!("Attempt to read from non-valid memory address {}", address);
        }
    }
    pub fn get_mut(&mut self, address: MemoryAddress) -> &mut PyObject {
        let cell = &self.memory[address];
        if cell.valid {
            return &mut cell.data;
        } else {
            panic!("Attempt to read from non-valid memory address {}", address);
        }
    }

    pub fn write<T: Any + Debug>(&mut self, address: MemoryAddress, data: PyObject) {
        let cell = &mut self.memory[address];
        debug_assert!(!cell.is_const);
        if cell.valid {
            cell.data = data;
        } else {
            panic!("Attempt to write in non-valid memory address {}", address);
        }
    }

    pub fn make_const(&mut self, address: MemoryAddress) {
        let cell = &mut self.memory[address];
        cell.is_const = true;
    }

    pub fn deallocate(&mut self, address: MemoryAddress) {
        let cell = &mut self.memory[address];
        if cell.is_const { return };
        if cell.valid {
            cell.valid = false;
        } else {
            panic!("Attempt to dealloate already invalid memory at address in non-valid memory address {}", address)
        }
        
        self.recently_deallocated_indexes.push(address);
    }

    pub fn allocate(&self) -> MemoryAddress {
        let dealloc = self.recently_deallocated_indexes.pop();

        match dealloc {
            Some(address) => {
                let mut cell = &mut self.memory[address];
                debug_assert!(!cell.is_const);
                if cell.valid {
                    panic!(
                        "Attempt to allocate onto already occupied address {}",
                        address
                    )
                } else {
                    unsafe {
                        let ptr = &mut *cell.data.get();
                        *ptr = CellValue::Uninitialized
                    }
                    cell.valid = true;
                }
                return address;
            }
            None => {
                let mut borrow = self.memory.borrow_mut();
                borrow.push(MemoryCell {
                    data: UnsafeCell::new(CellValue::Uninitialized),
                    valid: true,
                    is_const: false,
                });
                return borrow.len() - 1;
            }
        };
    }

    pub fn allocate_and_write<T: Any + Debug>(&self, data: T) -> MemoryAddress {
        let address = self.allocate();
        self.write(address, data);
        return address;
    }

    pub fn get_statistics(&self) -> MemoryStatistics {
        let mem = self.memory.borrow();
        let allocated_slots = mem.len();
        let slots_in_use = mem.iter().filter(|x| x.valid).count();
        MemoryStatistics {
            allocated_slots, slots_in_use
        }
    }
}

pub struct StackFrame {
    pub values: HashMap<String, MemoryAddress>,
    pub stack: Vec<MemoryAddress>,
    pub current_function: MemoryAddress
}

#[derive(PartialEq, Eq, Hash)]
pub enum SpecialValue {
    Type,
    NoneType,
    NoneValue,
    NotImplementedType,
    NotImplementedValue,
    CallableType,
    ModuleType, 
}

pub struct BuiltinTypeAddresses {
    pub int: MemoryAddress,
    pub float: MemoryAddress,
    pub boolean: MemoryAddress,
    pub string: MemoryAddress,
    pub true_val: MemoryAddress,
    pub false_val: MemoryAddress
}

pub struct Runtime {
    pub program_consts: RefCell<Vec<MemoryAddress>>,
    pub stack: RefCell<Vec<StackFrame>>,
    pub memory: Memory,
    pub builtin_type_addrs: BuiltinTypeAddresses,
    pub special_values: HashMap<SpecialValue, MemoryAddress>,
    pub modules: RefCell<HashMap<String, MemoryAddress>>,
    pub prog_counter: Cell<usize>
}

impl Runtime {
    pub fn new() -> Runtime {
        let mut interpreter = Runtime {
            stack: RefCell::new(vec![StackFrame{
                values: HashMap::new(),
                current_function: 0,
                stack: vec![]
            }]),
            memory: Memory {
               // gc_graph: HashMap::new(),
                memory: RefCell::new(vec![]),
                recently_deallocated_indexes: RefCell::new(vec![]),
            },
            special_values: HashMap::new(),
            modules: RefCell::new(HashMap::new()),
            program_consts: RefCell::new(vec![]),
            prog_counter: Cell::new(0),
            builtin_type_addrs: BuiltinTypeAddresses {
                int: 0,
                float: 0, 
                boolean: 0,
                string: 0,
                true_val: 0,
                false_val: 0
            }
        };

        let type_type = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: 0,
            structure: PyObjectStructure::Type {
                name: String::from("type"),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new(),
                supertype: None
            },
        }));

        let none_type = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: type_type,
            structure: PyObjectStructure::Type {
                name: String::from("NoneType"),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new(),
                supertype: None
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
                unbounded_functions: HashMap::new(),
                supertype: None
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
                unbounded_functions: HashMap::new(),
                supertype: None
            },
        }));

        let module_type = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: type_type,
            structure: PyObjectStructure::Type {
                name: String::from("module"),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new(),
                supertype: None
            },
        }));

        let builtin_module_obj = interpreter.allocate_and_write(Box::new(PyObject {
            type_addr: module_type,
            structure: PyObjectStructure::Module {
                name: BUILTIN_MODULE.to_string(),
                objects: HashMap::new()
            },
        }));

        interpreter.make_const(type_type);
        interpreter.make_const(none_type);
        interpreter.make_const(none_value);
        interpreter.make_const(not_implemented_type);
        interpreter.make_const(not_implemented_value);
        interpreter.make_const(callable_type);
        interpreter.make_const(module_type);

        interpreter.special_values.insert(SpecialValue::Type, type_type);
        interpreter.special_values.insert(SpecialValue::NoneType, none_type);
        interpreter.special_values.insert(SpecialValue::NoneValue, none_value);
        interpreter.special_values.insert(SpecialValue::NotImplementedType, not_implemented_type);
        interpreter.special_values.insert(SpecialValue::NotImplementedValue, not_implemented_value);
        interpreter.special_values.insert(SpecialValue::CallableType, callable_type);
        interpreter.special_values.insert(SpecialValue::ModuleType, module_type);

        interpreter.modules.borrow_mut().insert(BUILTIN_MODULE.to_string(), builtin_module_obj);

        return interpreter;
    }

    pub fn store_const(&self, value: MemoryAddress) {
        self.program_consts.borrow_mut().push(value);
        self.make_const(value);
    }

    pub fn get_const(&self, index: usize) -> MemoryAddress {
        return *self.program_consts.borrow().get(index).unwrap()
    }

    pub fn create_type(
        &self,
        module: &str,
        name: &str,
        supertype: Option<MemoryAddress>
    ) -> MemoryAddress {
        let created_type = PyObject {
            type_addr: self.special_values[&SpecialValue::Type],
            structure: PyObjectStructure::Type {
                name: name.to_string(),
                bounded_functions: HashMap::new(),
                unbounded_functions: HashMap::new(),
                supertype
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

    pub fn increase_refcount(&self, addr: MemoryAddress) {
        let pyobj = self.get_pyobj_byaddr(addr).unwrap();
        if let PyObjectStructure::Object {raw_data: _, refcount} = &mut pyobj.structure {
            *refcount = *refcount + 1;
        }
    }

    pub fn decrease_refcount(&self, addr: MemoryAddress) {
        let pyobj = self.get_pyobj_byaddr(addr).unwrap();
        if let PyObjectStructure::Object {raw_data, refcount} = &mut pyobj.structure {
            if *refcount > 0 {
                *refcount = *refcount - 1;
            }
            if *refcount <= 1 {
                self.memory.deallocate(addr);
                self.memory.deallocate(*raw_data);
            }
        }
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
                    unbounded_functions: _,
                    supertype
                } => {
                    match bounded_functions.get(method_name) {
                        Some(addr) => Some(*addr),
                        None => match supertype {
                            Some(supertype_addr) => self.get_type_method_addr_byname(*supertype_addr, method_name),
                            None => None
                        }
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
                    unbounded_functions: _,
                    supertype: _
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

    pub fn get_pyobj_type_addr(&self, addr: MemoryAddress) -> MemoryAddress {
        match self.get_pyobj_byaddr(addr) {
            Some(obj) => {
                return obj.type_addr;
            }
            None => {
                panic!("Attempt to get type addr on non-existing object");
            }
        }
    }

    pub fn get_pyobj_type_name(&self, addr: MemoryAddress) -> &str {
        let type_addr = self.get_pyobj_type_addr(addr);
        return self.get_type_name(type_addr);
    }

    pub fn allocate_module_type_byname_raw<T: Any + Debug>(&self, module: &str, name: &str, raw_data: T) -> MemoryAddress {
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

    pub fn make_const(&self, addr: MemoryAddress) {
        self.memory.make_const(addr);
        let pyobj = self.get_pyobj_byaddr(addr).unwrap();
        if let PyObjectStructure::Object{raw_data, refcount: _} = pyobj.structure {
            self.memory.make_const(raw_data);
        }
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

    pub fn allocate_type_byaddr_raw<T: Any + Debug>(
        &self,
        type_addr: MemoryAddress,
        raw_data: T,
    ) -> MemoryAddress {
        let raw_data_ptr = self.allocate_and_write(raw_data);

        return self.allocate_type_byaddr_raw_struct(
            type_addr,
            PyObjectStructure::Object {
                raw_data: raw_data_ptr,
                refcount: 1
            },
        );
    }

    pub fn create_callable_pyobj(&self, callable: PyCallable, name: Option<String>) -> MemoryAddress {
        let raw_callable = self.allocate_and_write(Box::new(callable));

        self.allocate_type_byaddr_raw_struct(
            self.special_values[&SpecialValue::CallableType],
            PyObjectStructure::Callable { code: raw_callable, name },
        )
    }

    pub fn register_bounded_func<F>(
        &self, 
        module_name: &str,
        type_name: &str,
        name: &str,
        callable: F) where F: Fn(&Runtime, CallParams) -> MemoryAddress + 'static {
        let pycallable = PyCallable {
            code: Box::new(callable)
        };
        let func_addr = self.create_callable_pyobj(pycallable, Some(name.to_string()));
        let module = self.find_in_module(module_name, type_name).unwrap();
        let pyobj = self.get_pyobj_byaddr(module).unwrap();
        if let PyObjectStructure::Type{ name: _, bounded_functions, unbounded_functions: _, supertype: _ } = &mut pyobj.structure {
            bounded_functions.insert(name.to_string(), func_addr);
        } else {
            panic!("Object is not a type: {}.{}", module_name, type_name);
        }
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
            refcount: _
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
            let raw_callable = self.get_rawdata_byaddr(*code);
            let pyobj_callable = raw_callable
                .downcast_ref::<PyCallable>()
                .unwrap();
            return (pyobj_callable.code)(self, call_params);
        } else {
            panic!("Object is not callable: {:?}", callable_pyobj);
        }
    }

    pub fn call_method(&self, addr: MemoryAddress, method_name: &str, params: &[MemoryAddress]) -> Option<MemoryAddress> {
        let pyobj = self.get_pyobj_byaddr(addr).unwrap();
        self.get_type_method_addr_byname(pyobj.type_addr, method_name).map(move |method_addr| {
            self.callable_call(method_addr, CallParams {
                bound_pyobj: Some(addr),
                func_address: method_addr,
                func_name: Some(method_name),
                params
            })
        })
    }

    pub fn bounded_function_call_byaddr(
        &self,
        bound_obj_addr: MemoryAddress,
        method_addr: MemoryAddress,
        params: &[MemoryAddress],
    ) -> MemoryAddress {
        let pyobj: &PyObject = self.get_pyobj_byaddr(bound_obj_addr).unwrap();
        let type_addr = pyobj.type_addr;
        let type_pyobj = self.get_pyobj_byaddr(type_addr).unwrap();

        //type_pyobj must be a type
        if let PyObjectStructure::Type {
            name: _,
            bounded_functions: _,
            unbounded_functions: _,
            supertype: _
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
                                Some(fname) => Some(fname),
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
        params: &[MemoryAddress],
    ) -> MemoryAddress {
        let unbounded_func = self.get_pyobj_byaddr(function_addr);
        if let Some(unbounded_func_pyobj) = unbounded_func {
            return match &unbounded_func_pyobj.structure {
                PyObjectStructure::Callable{code: _, name} => {
                    let call_params = CallParams {
                        bound_pyobj: None,
                        func_address: function_addr,
                        func_name: match name {
                            Some(fname) => Some(fname),
                            None => None
                        },
                        params,
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

    pub fn pop_stack(&self) -> MemoryAddress {
        match self.stack.borrow_mut().last_mut().unwrap().stack.pop() {
            Some(addr) => addr,
            None => panic!("Attempt to pop on empty stack!")
        }
    }

    pub fn top_stack(&self) -> MemoryAddress {
        match self.stack.borrow().last().unwrap().stack.last() {
            Some(addr) => *addr,
            None => panic!("Attempt to get top of stack on empty stack!")
        }
    }

    pub fn push_onto_stack(&self, value: MemoryAddress) {
        self.stack.borrow_mut().last_mut().unwrap().stack.push(value)
    }

    pub fn pop_stack_frame(&self) -> StackFrame {
        let mut current_stack = self.stack.borrow_mut();
        match current_stack.pop() {
            Some(stack_frame) => {
                for addr in stack_frame.stack.iter() {
                    self.decrease_refcount(*addr)
                }
                stack_frame
            },
            None => panic!("Attempt to pop on empty stack frames!")
        }
    }

    pub fn bind_local(&self, name: &str, addr: MemoryAddress) {
        let mut current_stack = self.stack.borrow_mut();
        let current_frame = current_stack.last_mut().unwrap();
        current_frame.values.insert(name.to_string(), addr);
    }
  
    pub fn get_local(&self, name: &str) -> Option<MemoryAddress> {
        let mut current_stack = self.stack.borrow_mut();
        let current_frame = current_stack.last_mut().unwrap();
        current_frame.values.get(name).map(|a| *a)
    }

    pub fn allocate_and_write<'a, T: Any + Debug>(&'a self, data: T) -> MemoryAddress {
        self.memory.allocate_and_write(data)
    }

    pub fn get_pc(&self) -> usize {
        self.prog_counter.get()
    }

    pub fn jump_pc(&self, delta: isize) -> usize {
        self.prog_counter.set((self.prog_counter.get() as isize + delta) as usize);
        self.prog_counter.get()
    }

    pub fn set_pc(&self, pc: usize) -> usize {
        self.prog_counter.set(pc);
        self.prog_counter.get()
    }
}

macro_rules! check_builtin_func_params {
    ($name:expr, $expected:expr, $received:expr) => {
        if $expected != $received {
            panic!(
                "{}() expected {} arguments, got {}",
                $name, $expected, $received
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_types::{register_builtins};

    #[test]
    fn simply_instantiate_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let pyobj_int_addr = interpreter.allocate_type_byname_raw("int", Box::new(1 as i128));
        let result_value = interpreter.get_raw_data_of_pyobj::<i128>(pyobj_int_addr);
        assert_eq!(1, *result_value);
    }

    #[test]
    fn simply_instantiate_bool() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let pyobj_int_addr = interpreter.allocate_type_byname_raw("bool", Box::new(1 as i128));
        let result_value = interpreter.get_raw_data_of_pyobj::<i128>(pyobj_int_addr);
        assert_eq!(1, *result_value);
    }

    #[test]
    fn call_int_add_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_type_byname_raw("int", Box::new(1 as i128));
        let number2 = interpreter.allocate_type_byname_raw("int", Box::new(3 as i128));

        //number1.__add__(number2)
        let result = interpreter.call_method(number1, "__add__", &[number2]).unwrap();
        let result_value = *interpreter.get_raw_data_of_pyobj::<i128>(result);

        assert_eq!(result_value, 4);
    }

    #[test]
    fn call_bool_add_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_type_byname_raw("bool", Box::new(1 as i128));
        let number2 = interpreter.allocate_type_byname_raw("int", Box::new(3 as i128));

        //number1.__add__(number2)
        let result = interpreter.call_method(number1, "__add__", &[number2]).unwrap();
        let result_value = *interpreter.get_raw_data_of_pyobj::<i128>(result);

        assert_eq!(result_value, 4);
    }

    #[test]
    fn call_int_add_float() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_type_byname_raw("int", Box::new(1 as i128));
        let number2 = interpreter.allocate_type_byname_raw("float", Box::new(3.5 as f64));

        //number1.__add__(number2)
        let result = interpreter.call_method(number1, "__add__", &[number2]).unwrap();
        let result_value = *interpreter.get_raw_data_of_pyobj::<f64>(result);

        assert_eq!(result_value, 4.5 as f64);
    }

    #[test]
    fn call_float_add_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_type_byname_raw("float", Box::new(3.5 as f64));
        let number2 = interpreter.allocate_type_byname_raw("int", Box::new(1 as i128));

        //number1.__add__(number2)
        let result = interpreter.call_method(number1, "__add__", &[number2]).unwrap();
        let result_value = *interpreter.get_raw_data_of_pyobj::<f64>(result);

        assert_eq!(result_value, 4.5 as f64);
    }


    #[test]
    fn call_float_add_float() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_type_byname_raw("float", Box::new(3.4 as f64));
        let number2 = interpreter.allocate_type_byname_raw("float", Box::new(1.1 as f64));

        //number1.__add__(number2)
        let result = interpreter.call_method(number1, "__add__", &[number2]).unwrap();
        let result_value = *interpreter.get_raw_data_of_pyobj::<f64>(result);

        assert_eq!(result_value, 4.5 as f64);
    }

    #[test]
    fn call_float_mul_float() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_type_byname_raw("float", Box::new(2.0 as f64));
        let number2 = interpreter.allocate_type_byname_raw("float", Box::new(3.0 as f64));

        //number1.__add__(number2)
        let result = interpreter.call_method(number1, "__mul__", &[number2]).unwrap();
        let result_value = *interpreter.get_raw_data_of_pyobj::<f64>(result);

        assert_eq!(result_value, 6.0 as f64);
    }

    #[test]
    fn call_int_mul_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_type_byname_raw("int", Box::new(2 as i128));
        let number2 = interpreter.allocate_type_byname_raw("int", Box::new(3 as i128));

        //number1.__add__(number2)
        let result = interpreter.call_method(number1, "__mul__", &[number2]).unwrap();
        let result_value = *interpreter.get_raw_data_of_pyobj::<i128>(result);

        assert_eq!(result_value, 6 as i128);
    }

    #[test]
    fn bind_local_test() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number = interpreter.allocate_type_byname_raw("int", Box::new(17 as i128));
        interpreter.bind_local("x", number);

        let addr_local = interpreter.get_local("x").unwrap();
        let result_value = *interpreter.get_raw_data_of_pyobj::<i128>(addr_local);

        assert_eq!(result_value, 17 as i128);
    }

 
}
