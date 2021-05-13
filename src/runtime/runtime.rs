use std::cell::Cell;
use std::collections::HashMap;
use crate::bytecode::program::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;

/* this is done by somewhat following the python data model in https://docs.python.org/3/reference/datamodel.html */


pub struct PyCallable {
    pub code: Box<dyn Fn(&Runtime, CallParams) -> MemoryAddress>,
}

impl std::fmt::Debug for PyCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[rust native code]")
    }
}

pub struct CallParams<'a> {
    pub bound_pyobj: Option<MemoryAddress>,
    pub func_address: MemoryAddress,
    pub func_name: Option<&'a str>,
    pub params: &'a [MemoryAddress],
}

use std::cell::RefCell;

pub struct StackFrame {
    pub function_name: String,
    pub values: Vec<MemoryAddress>,
    pub stack: Vec<MemoryAddress>,
    pub exception: Option<MemoryAddress>,
    pub prog_counter: Cell<usize>,
}

#[derive(PartialEq, Eq, Hash)]
pub enum SpecialValue {
    Type,
    NoneType,
    NoneValue,
    NotImplementedType,
    NotImplementedValue,
    StopIterationType,
    StopIterationValue,
    CallableType,
    ModuleType,
}

pub struct BuiltinTypeAddresses {
    pub int: MemoryAddress,
    pub float: MemoryAddress,
    pub boolean: MemoryAddress,
    pub string: MemoryAddress,
    pub list: MemoryAddress,
    pub index_err: MemoryAddress,
    pub code_object: MemoryAddress,
    pub true_val: MemoryAddress,
    pub false_val: MemoryAddress,
}

pub struct Runtime {
    pub stack: RefCell<Vec<StackFrame>>,
    pub memory: UnsafeMemory,
    pub builtin_type_addrs: BuiltinTypeAddresses,
    pub special_values: HashMap<SpecialValue, MemoryAddress>,
    pub modules: HashMap<String, MemoryAddress>,
    pub builtin_names: Vec<MemoryAddress>,
}

impl Runtime {
  
    pub fn new() -> Runtime {
        let memory = UnsafeMemory::new();
        let nullptr = memory.null_ptr();
        let mut interpreter = Runtime {
            stack: RefCell::new(vec![StackFrame {
                function_name: "__main__".to_owned(),
                values: vec![],
                stack: vec![],
                exception: None,
                prog_counter: Cell::new(0),
            }]),
            memory: memory,
            special_values: HashMap::new(),
            modules: HashMap::new(),
            builtin_names: vec![],
            builtin_type_addrs: BuiltinTypeAddresses {
                int: nullptr,
                float: nullptr,
                boolean: nullptr,
                string: nullptr,
                list: nullptr,
                true_val: nullptr,
                false_val: nullptr,
                index_err: nullptr,
                code_object: nullptr
            },
        };

        let type_type = interpreter.allocate_and_write(PyObject {
            type_addr: nullptr,
            properties: HashMap::new(),
            structure: PyObjectStructure::Type {
                name: String::from("type"),
                methods: HashMap::new(),
                functions: HashMap::new(),
                supertype: None,
            },
            is_const: false,
        });

        let none_type = interpreter.allocate_and_write(PyObject {
            type_addr: type_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Type {
                name: String::from("NoneType"),
                methods: HashMap::new(),
                functions: HashMap::new(),
                supertype: None,
            },
            is_const: false,
        });

        let none_value = interpreter.allocate_and_write(PyObject {
            type_addr: none_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::None,
            is_const: false,
        });

        let not_implemented_type = interpreter.allocate_and_write(PyObject {
            type_addr: type_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Type {
                name: String::from("NotImplemented"),
                methods: HashMap::new(),
                functions: HashMap::new(),
                supertype: None,
            },
            is_const: false,
        });

        let not_implemented_value = interpreter.allocate_and_write(PyObject {
            type_addr: not_implemented_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::NotImplemented,
            is_const: false,
        });

        let stop_iteration_type = interpreter.allocate_and_write(PyObject {
            type_addr: type_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Type {
                name: String::from("StopIteration"),
                methods: HashMap::new(),
                functions: HashMap::new(),
                supertype: None,
            },
            is_const: false,
        });

        let stop_iteration_value = interpreter.allocate_and_write(PyObject {
            type_addr: stop_iteration_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::NotImplemented,
            is_const: false,
        });

        let callable_type = interpreter.allocate_and_write(PyObject {
            type_addr: type_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Type {
                name: String::from("function"),
                methods: HashMap::new(),
                functions: HashMap::new(),
                supertype: None,
            },
            is_const: false,
        });

        let module_type = interpreter.allocate_and_write(PyObject {
            type_addr: type_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Type {
                name: String::from("module"),
                methods: HashMap::new(),
                functions: HashMap::new(),
                supertype: None,
            },
            is_const: false,
        });

        let builtin_module_obj = interpreter.allocate_and_write(PyObject {
            type_addr: module_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Module {
                name: BUILTIN_MODULE.to_string(),
                objects: HashMap::new(),
            },
            is_const: false,
        });

        let main_module_obj = interpreter.allocate_and_write(PyObject {
            type_addr: module_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Module {
                name: MAIN_MODULE.to_string(),
                objects: HashMap::new(),
            },
            is_const: false,
        });


        interpreter.make_const(type_type);
        interpreter.make_const(none_type);
        interpreter.make_const(none_value);
        interpreter.make_const(not_implemented_type);
        interpreter.make_const(not_implemented_value);
        interpreter.make_const(stop_iteration_type);
        interpreter.make_const(stop_iteration_value);
        interpreter.make_const(callable_type);
        interpreter.make_const(module_type);
        interpreter.make_const(main_module_obj);

        interpreter
            .special_values
            .insert(SpecialValue::Type, type_type);
        interpreter
            .special_values
            .insert(SpecialValue::NoneType, none_type);
        interpreter
            .special_values
            .insert(SpecialValue::NoneValue, none_value);
        interpreter
            .special_values
            .insert(SpecialValue::NotImplementedType, not_implemented_type);
        interpreter
            .special_values
            .insert(SpecialValue::NotImplementedValue, not_implemented_value);
        interpreter
            .special_values
            .insert(SpecialValue::StopIterationType, stop_iteration_type);
        interpreter
            .special_values
            .insert(SpecialValue::StopIterationValue, stop_iteration_value);
        interpreter
            .special_values
            .insert(SpecialValue::CallableType, callable_type);
        interpreter
            .special_values
            .insert(SpecialValue::ModuleType, module_type);

        interpreter
            .modules
            .insert(BUILTIN_MODULE.to_string(), builtin_module_obj);

        interpreter
            .modules
            .insert(MAIN_MODULE.to_string(), main_module_obj);

        return interpreter;
    }

    pub fn clear_stacks(&mut self) {
        {
            let mut stack_borrow = self.stack.borrow_mut();
            while stack_borrow.len() > 0 {
                let popped = stack_borrow.pop().unwrap();
                for stack_val in popped.stack.iter() {
                    self.decrease_refcount(*stack_val);
                } 
            }
        }
        self.new_stack_frame("__main__".to_owned());
    }

    pub fn create_type(
        &mut self,
        module: &str,
        name: &str,
        supertype: Option<MemoryAddress>,
    ) -> MemoryAddress {
        let created_type = PyObject {
            properties: HashMap::new(),
            type_addr: self.special_values[&SpecialValue::Type],
            structure: PyObjectStructure::Type {
                name: name.to_string(),
                methods: HashMap::new(),
                functions: HashMap::new(),
                supertype,
            },
            is_const: false,
        };
        let type_address = self.allocate_and_write(created_type);
        let module_addr = *self.modules.get(module).unwrap();
        let pyobj = self.get_pyobj_byaddr_mut(module_addr);
        match &mut pyobj.structure {
            PyObjectStructure::Module { name: _, objects } => match objects.get(name) {
                Some(_) => {
                    panic!("Name already exists in module {}: {}", module, name);
                }
                None => {
                    objects.insert(name.to_string(), type_address);
                    return type_address;
                }
            },
            _ => {
                panic!(
                    "Module name {} was found but it's not actually a module",
                    module
                );
            }
        }
    }

    pub fn add_to_module(&self, module: &str, name: &str, pyobject_addr: MemoryAddress) {
        let module_addr = *self.modules.get(module).unwrap();
        let pyobj = self.get_pyobj_byaddr_mut(module_addr);
        match &mut pyobj.structure {
            PyObjectStructure::Module { name: _, objects } => match objects.get(name) {
                Some(_) => {
                    panic!("Name already exists in module {}: {}", module, name);
                }
                None => {
                    objects.insert(name.to_string(), pyobject_addr);
                }
            },
            _ => {
                panic!(
                    "Module name {} was found but it's not actually a module",
                    module
                );
            }
        }
    }

    pub fn find_module(&self, module: &str) -> Option<MemoryAddress> {
        return self.modules.get(module).map(|addr: &MemoryAddress| *addr);
    }

    pub fn find_in_module(&self, module: &str, name: &str) -> Option<MemoryAddress> {
        let module_addr = self.find_module(module).unwrap();
        return self.find_in_module_addr(module_addr, name);
    }

    pub fn find_in_module_addr(
        &self,
        module_addr: MemoryAddress,
        name: &str,
    ) -> Option<MemoryAddress> {
        let module_pyobj = self.get_pyobj_byaddr(module_addr);
        match &module_pyobj.structure {
            PyObjectStructure::Module { name: _, objects } => objects.get(name).map(|addr| *addr),
            _ => panic!("Object is not module: {:?}", module_pyobj.structure),
        }
    }

    pub fn get_obj_property(&self, addr: MemoryAddress, attr_name: &str) -> Option<MemoryAddress> {
        let pyobj = self.get_pyobj_byaddr(addr);
        return pyobj.properties.get(attr_name).map(|addr| *addr);
    }

    pub fn get_pyobj_byaddr(&self, addr: MemoryAddress) -> &PyObject {
        return self.memory.get(addr);
    }

    pub fn get_pyobj_byaddr_mut(&self, addr: MemoryAddress) -> &mut PyObject {
        return self.memory.get_mut(addr);
    }

    pub fn get_refcount(&self, addr: MemoryAddress) -> i32 {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::Object {
            raw_data: _,
            refcount,
        } = &mut pyobj.structure
        {
            return (*refcount) as i32;
        } else {
            return -1
        }
    }

    pub fn get_function_name(&self, addr: MemoryAddress) -> String {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::NativeCallable {
            code: _,
            name,
        } = &mut pyobj.structure
        {
            if let Some(n) = name {
                return n.clone();
            }
            else {
                return "unknown".to_owned();
            }
        } else if let PyObjectStructure::UserDefinedFunction {
            code: _,
            name,
        } = &mut pyobj.structure {
            
            return name.to_owned();
        } else {
            return "unknown".to_owned();
        }
    }

    pub fn increase_refcount(&self, addr: MemoryAddress) {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::Object {
            raw_data: _,
            refcount,
        } = &mut pyobj.structure
        {
            *refcount = *refcount + 1;
        }
    }

    pub fn decrease_refcount(&self, addr: MemoryAddress) {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::Object {
            raw_data: _,
            refcount,
        } = &mut pyobj.structure
        {
            if *refcount > 0 {
                *refcount = *refcount - 1;
            }

            if *refcount <= 0 {
                self.memory.deallocate(addr);
            }
        }
    }

    pub fn get_method_addr_byname(
        &self,
        type_addr: MemoryAddress,
        method_name: &str,
    ) -> Option<MemoryAddress> {
        let pyobj = self.get_pyobj_byaddr(type_addr);
        match &pyobj.structure {
            PyObjectStructure::Type {
                name: _,
                methods,
                functions: _,

                supertype,
            } => match methods.get(method_name) {
                Some(addr) => Some(*addr),
                None => match supertype {
                    Some(supertype_addr) => {
                        self.get_method_addr_byname(*supertype_addr, method_name)
                    }
                    None => None,
                },
            },
            _ => None,
        }
    }

    pub fn get_type_name(&self, addr: MemoryAddress) -> &str {
        let pyobj = self.get_pyobj_byaddr(addr);
        match &pyobj.structure {
            PyObjectStructure::Type {
                name,
                methods: _,
                functions: _,

                supertype: _,
            } => {
                return &name;
            }
            _ => {
                panic!(
                    "Attempt to get type name on a non-type object {:?}",
                    pyobj.structure
                );
            }
        }
    }

    pub fn get_pyobj_type_addr(&self, addr: MemoryAddress) -> MemoryAddress {
        let pyobj = self.get_pyobj_byaddr(addr);
        return pyobj.type_addr;
    }

    pub fn get_pyobj_type_name(&self, addr: MemoryAddress) -> &str {
        let type_addr = self.get_pyobj_type_addr(addr);
        return self.get_type_name(type_addr);
    }

    pub fn make_const(&self, addr: MemoryAddress) {
        self.memory.make_const(addr);
    }


    pub fn allocate_user_defined_function(
        &self,
        code: CodeObjectContext,
        name: String
    ) -> MemoryAddress {
        let obj = PyObject {
            properties: HashMap::new(),
            type_addr: self.builtin_type_addrs.code_object,
            structure: PyObjectStructure::UserDefinedFunction {code, name},
            is_const: true,
        };
        return self.allocate_and_write(obj);
    }


    pub fn allocate_type_byaddr_raw_struct(
        &self,
        type_addr: MemoryAddress,
        structure: PyObjectStructure,
    ) -> MemoryAddress {
        let obj = PyObject {
            properties: HashMap::new(),
            type_addr,
            structure,
            is_const: false,
        };
        return self.allocate_and_write(obj);
    }

    pub fn allocate_builtin_type_byname_raw(
        &self,
        type_name: &str,
        raw_data: BuiltInTypeData,
    ) -> MemoryAddress {
        let type_addr = self.find_in_module(BUILTIN_MODULE, type_name).unwrap();
        self.allocate_type_byaddr_raw(type_addr, raw_data)
    }

    pub fn allocate_type_byaddr_raw(
        &self,
        type_addr: MemoryAddress,
        raw_data: BuiltInTypeData,
    ) -> MemoryAddress {
        return self.memory.allocate_and_write_builtin(type_addr, raw_data);
    }

    pub fn create_callable_pyobj(
        &self,
        callable: PyCallable,
        name: Option<String>,
    ) -> MemoryAddress {
        self.allocate_type_byaddr_raw_struct(
            self.special_values[&SpecialValue::CallableType],
            PyObjectStructure::NativeCallable {
                code: callable,
                name,
            },
        )
    }

    pub fn register_type_unbounded_func<F>(
        &mut self,
        module_name: &str,
        type_name: &str,
        name: &str,
        callable: F,
    ) where
        F: Fn(&Runtime, CallParams) -> MemoryAddress + 'static {
            let pycallable = PyCallable {
                code: Box::new(callable),
            };
            let func_addr = self.create_callable_pyobj(pycallable, Some(name.to_string()));
            let module = self.find_in_module(module_name, type_name).unwrap();
            let pyobj_type = self.get_pyobj_byaddr_mut(module);
            if let PyObjectStructure::Type {
                name: _,
                methods: _,
                functions,
                supertype: _,
            } = &mut pyobj_type.structure
            {
                functions.insert(name.to_string(), func_addr);
            } else {
                panic!("Object is not a type: {}.{}", module_name, type_name);
            }
    }

    pub fn register_bounded_func<F>(
        &mut self,
        module_name: &str,
        type_name: &str,
        name: &str,
        callable: F,
    ) where
        F: Fn(&Runtime, CallParams) -> MemoryAddress + 'static,
    {   let type_addr = self.find_in_module(module_name, type_name).unwrap();
        self.register_bounded_func_on_addr(type_addr, name, callable);
    }

    pub fn register_bounded_func_on_addr<F>(
        &mut self,
        type_addr: MemoryAddress,
        name: &str,
        callable: F,
    ) where
        F: Fn(&Runtime, CallParams) -> MemoryAddress + 'static,
    {
        let pycallable = PyCallable {
            code: Box::new(callable),
        };
        let func_addr = self.create_callable_pyobj(pycallable, Some(name.to_string()));
        let pyobj_type = self.get_pyobj_byaddr_mut(type_addr);
        if let PyObjectStructure::Type {
            name: _,
            methods,
            functions: _,
            supertype: _,
        } = &mut pyobj_type.structure
        {
            methods.insert(name.to_string(), func_addr);
        } else {
            panic!("Object is not a type: {:?}", pyobj_type);
        }
    }


    pub fn get_raw_data_of_pyobj(&self, addr: MemoryAddress) -> &BuiltInTypeData {
        let pyobj = self.get_pyobj_byaddr(addr);
        if let PyObjectStructure::Object {
            raw_data,
            refcount: _,
        } = &pyobj.structure
        {
            return &raw_data;
        } else {
            panic!(
                "get_raw_data_of_pyobj cannot be called on {:?} {:p}",
                pyobj.structure, addr
            )
        }
    }

    pub fn get_function_bytecode(&self, addr: MemoryAddress) -> &[Instruction] {
        let pyobj = self.get_pyobj_byaddr(addr);
        if let PyObjectStructure::UserDefinedFunction { code, name: _ } = &pyobj.structure {
            return &code.code.instructions;
        } else {
            panic!(
                "get_raw_data_of_pyobj cannot be called on {:?}",
                pyobj.structure
            )
        }
    }

    pub fn get_raw_data_of_pyobj_mut(&self, addr: MemoryAddress) -> &mut BuiltInTypeData {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::Object {
            raw_data,
            refcount: _,
        } = &mut pyobj.structure
        {
            return raw_data;
        } else {
            panic!("get_raw_data_of_pyobj_mut cannot be called non-object")
        }
    }

    pub fn callable_call(
        &self,
        callable_addr: MemoryAddress,
        call_params: CallParams,
    ) -> MemoryAddress {
        let callable_pyobj = self.get_pyobj_byaddr(callable_addr);

        if let PyObjectStructure::NativeCallable { code, name: _ } = &callable_pyobj.structure {
            return (code.code)(self, call_params);
        } else {
            panic!("Object is not callable: {:?}", callable_pyobj);
        }
    }

    pub fn call_method(
        &self,
        addr: MemoryAddress,
        method_name: &str,
        params: &[MemoryAddress],
    ) -> Option<MemoryAddress> {
        let pyobj = self.get_pyobj_byaddr(addr);
        self.get_method_addr_byname(pyobj.type_addr, method_name)
            .map(move |method_addr| {
                self.callable_call(
                    method_addr,
                    CallParams {
                        bound_pyobj: Some(addr),
                        func_address: method_addr,
                        func_name: Some(method_name),
                        params,
                    },
                )
            })
    }

    pub fn raise_exception(
        &self,
        exception_value_addr: MemoryAddress,
    ) {
        let mut stack = self.stack.borrow_mut();
        let top_stack_frame = stack.last_mut().unwrap();
        top_stack_frame.exception = Some(exception_value_addr)
    }

    pub fn bounded_function_call_byaddr(
        &self,
        bound_obj_addr: MemoryAddress,
        method_addr: MemoryAddress,
        params: &[MemoryAddress],
    ) -> MemoryAddress {
        let pyobj = self.get_pyobj_byaddr(bound_obj_addr);
        let type_addr = pyobj.type_addr;
        let type_pyobj = self.get_pyobj_byaddr(type_addr);

        //type_pyobj must be a type
        if let PyObjectStructure::Type {
            name: _,
            methods: _,
            functions: _,
            supertype: _,
        } = &type_pyobj.structure
        {
            let bounded_method_pyobj = self.get_pyobj_byaddr(method_addr);
            let call_params = match &bounded_method_pyobj.structure {
                PyObjectStructure::NativeCallable { code: _, name: _ } => CallParams {
                    bound_pyobj: Some(bound_obj_addr),
                    func_address: method_addr,
                    func_name: None,
                    params: params,
                },
                _ => {
                    panic!("Not a method at addr: {:?}", method_addr);
                }
            };

            return self.callable_call(method_addr, call_params);
        } else {
            panic!(
                "FATAL ERROR: pyobj addr {:?} was supposed to be a type, but it's something else: {:?}",
                type_addr, &type_pyobj
            );
        }
    }

    pub fn new_stack_frame(&self, function_name: String) {
        self.stack.borrow_mut().push(StackFrame {
            function_name: function_name,
            values: vec![],
            stack: vec![],
            exception: None,
            prog_counter: Cell::new(0)
        })
    }

    pub fn pop_stack(&self) -> MemoryAddress {
        match self.stack.borrow_mut().last_mut().unwrap().stack.pop() {
            Some(addr) => addr,
            None => panic!("Attempt to pop on empty stack!"),
        }
    }

    pub fn print_traceback(&self)  {
        println!("Traceback: ");
        for val in self.stack.borrow().iter().rev() {
            println!("\tat {} +{}", val.function_name, val.prog_counter.get())
        }
    }
    
    pub fn print_stack(&self)  {
        print!("Stack: [");
        for val in self.stack.borrow().last().unwrap().stack.iter().rev() {
            let pyobj = self.get_pyobj_byaddr(*val);
            if let PyObjectStructure::Object {
                raw_data: raw,
                refcount: _,
            } = &pyobj.structure
            {
                print!("{} at {:p}", raw.to_string(), val);
                print!("    ");
            } else {
                print!("[a function]");
                print!("    ");
            }
          
        }
        print!("]\n");
    }

    pub fn get_stack_offset(&self, offset: isize) -> MemoryAddress {
        let len = self.stack.borrow_mut().last().unwrap().stack.len() as isize;
        let index = len - 1 + offset;
        return self.stack.borrow_mut().last().unwrap().stack[index as usize];
    }

    pub fn top_stack(&self) -> MemoryAddress {
        return self.get_stack_offset(0);
    }

    pub fn push_onto_stack(&self, value: MemoryAddress) {
        //println!("Pushing onto stack: {:?}", unsafe{&*value});
        self.stack.borrow_mut().last_mut().unwrap().stack.push(value)
    }

    pub fn pop_stack_frame(&self) {
        let mut stack = self.stack.borrow_mut();
        match stack.pop() {
            Some(stack_frame) => {
                for addr in stack_frame.stack.iter() {
                    self.decrease_refcount(*addr)
                }
            }
            None => panic!("Attempt to pop on empty stack frames!"),
        }
    }

    pub fn bind_local(&self, name: usize, addr: MemoryAddress) {
        //println!("Binding local: {} to {:?}", name, unsafe{&*addr});
        let mut stack = self.stack.borrow_mut();
        let current_frame = stack.last_mut().unwrap();

        //@Todo horrible stuff here
        if current_frame.values.len() == name {
            current_frame.values.push(addr);
        } else if current_frame.values.len() > name {
            current_frame.values[name] = addr
        } else if current_frame.values.len() < name {
            while current_frame.values.len() < name {
                current_frame.values.push(self.memory.null_ptr());
            }
            current_frame.values.push(addr);
        }
    }

    pub fn get_local(&self, name: usize) -> Option<MemoryAddress> {
        let stack = self.stack.borrow();
        let current_frame = stack.last().unwrap();
        current_frame.values.get(name).map(|a| *a)
    }

    pub fn allocate_and_write(&self, data: PyObject) -> MemoryAddress {
        self.memory.allocate_and_write(data)
    }

    pub fn get_pc(&self) -> usize {
        self.stack.borrow().last().unwrap().prog_counter.get()
    }

    pub fn jump_pc(&self, delta: isize) {
        let stack = self.stack.borrow();
        let last_cell = &stack.last().unwrap().prog_counter;
        let current_pc = last_cell.get();
        last_cell.set((current_pc as isize + delta) as usize);
    }

    pub fn set_pc(&self, pc: usize) {
        self.stack.borrow().last().unwrap().prog_counter.set(pc);
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_types::register_builtins;

    use crate::commons::float::Float;
    #[test]
    fn simply_instantiate_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let pyobj_int_addr =
            interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1));
        let result_value = interpreter.get_raw_data_of_pyobj(pyobj_int_addr).take_int();
        assert_eq!(1, result_value);
    }

    #[test]
    fn simply_instantiate_bool() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let pyobj_int_addr =
            interpreter.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(1));
        let result_value = interpreter.get_raw_data_of_pyobj(pyobj_int_addr).take_int();
        assert_eq!(1, result_value);
    }

    #[test]
    fn call_int_add_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1));
        let number2 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(3));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", &[number2])
            .unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_int();

        assert_eq!(result_value, 4);
    }

    #[test]
    fn call_bool_add_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(1));
        let number2 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(3));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", &[number2])
            .unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_int();

        assert_eq!(result_value, 4);
    }

    #[test]
    fn call_int_add_float() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1));
        let number2 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(3.5)));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", &[number2])
            .unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_float();

        assert_eq!(result_value, 4.5);
    }

    #[test]
    fn call_float_add_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(3.5)));
        let number2 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", &[number2])
            .unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_float();

        assert_eq!(result_value, 4.5);
    }

    #[test]
    fn call_float_add_float() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(3.4)));
        let number2 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(1.1)));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", &[number2])
            .unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_float();

        assert_eq!(result_value, 4.5);
    }

    #[test]
    fn call_float_mul_float() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(2.0)));
        let number2 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(3.0)));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__mul__", &[number2])
            .unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_float();

        assert_eq!(result_value, 6.0);
    }

    #[test]
    fn call_int_mul_int() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(2));
        let number2 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(3));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__mul__", &[number2])
            .unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_int();

        assert_eq!(result_value, 6);
    }

    #[test]
    fn bind_local_test() {
        let mut interpreter = Runtime::new();
        register_builtins(&mut interpreter);
        let number = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(17));
        interpreter.bind_local(0, number);

        let addr_local = interpreter.get_local(0).unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(addr_local).take_int();

        assert_eq!(result_value, 17);
    }
}
