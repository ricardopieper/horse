use crate::float::Float;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Debug;

/* this is done by somewhat following the python data model in https://docs.python.org/3/reference/datamodel.html */

pub type MemoryAddress = *mut PyObject;

pub const BUILTIN_MODULE: &'static str = "__builtins__";

pub struct PyCallable {
    pub code: Box<dyn Fn(&mut Runtime, CallParams) -> MemoryAddress>,
}

impl std::fmt::Debug for PyCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[rust native code]")
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum BuiltInTypeData {
    Int(i128),
    Float(Float),
    String(String),
    List(Vec<MemoryAddress>),
}

impl BuiltInTypeData {
    pub fn take_float(&self) -> f64 {
        match self {
            BuiltInTypeData::Float(Float(f)) => *f,
            _ => panic!("Tried to transform something into float unexpectedly"),
        }
    }

    pub fn take_int(&self) -> i128 {
        match self {
            BuiltInTypeData::Int(i) => *i,
            _ => panic!("Tried to transform something into int unexpectedly"),
        }
    }

    pub fn take_string(&self) -> &String {
        match self {
            BuiltInTypeData::String(s) => s,
            _ => panic!("Tried to transform something into string unexpectedly"),
        }
    }

    pub fn take_list(&self) -> &Vec<MemoryAddress> {
        match self {
            BuiltInTypeData::List(s) => s,
            _ => panic!("Tried to transform something into string unexpectedly"),
        }
    }

    pub fn take_list_mut(&mut self) -> &mut Vec<MemoryAddress> {
        match self {
            BuiltInTypeData::List(s) => s,
            _ => panic!("Tried to transform something into string unexpectedly"),
        }
    }
}

#[derive(Debug)]
pub enum PyObjectStructure {
    None,
    NotImplemented,
    Object {
        raw_data: BuiltInTypeData,
        refcount: usize,
    },
    NativeCallable {
        code: PyCallable,
        name: Option<String>,
    },
    Type {
        name: String,
        methods: HashMap<String, MemoryAddress>,
        functions: HashMap<String, MemoryAddress>,
        supertype: Option<MemoryAddress>,
    },
    Module {
        name: String,
        objects: HashMap<String, MemoryAddress>,
    },
}

#[derive(Debug)]
pub struct PyObject {
    pub type_addr: MemoryAddress,
    pub structure: PyObjectStructure,
    pub properties: HashMap<String, MemoryAddress>,
    pub is_const: bool,
}

impl PyObject {
    pub fn try_get_builtin(&self) -> Option<&BuiltInTypeData> {
        match &self.structure {
            PyObjectStructure::Object {
                raw_data,
                refcount: _,
            } => {
                return Some(raw_data);
            }
            _ => {
                return None;
            }
        }
    }
}

pub struct CallParams<'a> {
    pub bound_pyobj: Option<MemoryAddress>,
    pub func_address: MemoryAddress,
    pub func_name: Option<&'a str>,
    pub params: &'a [MemoryAddress],
}

pub struct Memory {
    pub recently_deallocated_addr: Vec<MemoryAddress>,
}

impl Memory {
    pub fn get(&self, address: MemoryAddress) -> &PyObject {
        if address.is_null() {
            panic!("Attempt to read from non-valid memory address null");
        }
        return unsafe { &*address };
    }

    pub fn get_mut(&mut self, address: MemoryAddress) -> &mut PyObject {
        if address.is_null() {
            panic!("Attempt to read from non-valid memory address null");
        }
        return unsafe { &mut *address };
    }

    pub fn make_const(&mut self, address: MemoryAddress) {
        unsafe { (*address).is_const = true };
    }

    pub fn deallocate(&mut self, address: MemoryAddress) {
        if address.is_null() {
            panic!("Null pointer!");
        }
        unsafe {
            if (*address).is_const {
                return;
            }
        }
        //do not deallocate yet! Could make a new box from raw and let it go out of scope

        self.recently_deallocated_addr.push(address);
    }

    pub fn allocate_and_write(&mut self, data: PyObject) -> MemoryAddress {
        let dealloc = self.recently_deallocated_addr.pop();
        match dealloc {
            Some(address) => {
                unsafe {
                    *address = data;
                };
                return address;
            }
            None => {
                let boxed = Box::new(data);
                let mutref = Box::leak(boxed); //hehe
                return mutref as *mut PyObject;
            }
        }
    }

    pub fn allocate_and_write_builtin(
        &mut self,
        type_addr: MemoryAddress,
        data: BuiltInTypeData,
    ) -> MemoryAddress {
        let dealloc = self.recently_deallocated_addr.pop();
        match dealloc {
            Some(address) => {
                let py_obj = unsafe { &mut *address };
                debug_assert!(!py_obj.is_const);
                if let PyObjectStructure::Object { raw_data, refcount } = &mut py_obj.structure {
                    *raw_data = data;
                    *refcount = 0;
                } else {
                    *py_obj = PyObject {
                        type_addr,
                        structure: PyObjectStructure::Object {
                            raw_data: data,
                            refcount: 0,
                        },
                        properties: HashMap::new(),
                        is_const: false,
                    };
                }
                return address;
            }
            None => self.allocate_and_write(PyObject {
                type_addr,
                structure: PyObjectStructure::Object {
                    raw_data: data,
                    refcount: 0,
                },
                properties: HashMap::new(),
                is_const: false,
            }),
        }
    }
}

pub struct StackFrame {
    pub values: Vec<MemoryAddress>,
    pub stack: Vec<MemoryAddress>,
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
    pub list: MemoryAddress,
    pub true_val: MemoryAddress,
    pub false_val: MemoryAddress,
}

pub struct Runtime {
    pub program_consts: Vec<MemoryAddress>,
    pub stack: Vec<StackFrame>,
    pub memory: Memory,
    pub builtin_type_addrs: BuiltinTypeAddresses,
    pub special_values: HashMap<SpecialValue, MemoryAddress>,
    pub modules: HashMap<String, MemoryAddress>,
    pub prog_counter: Cell<usize>,
}

impl Runtime {
    pub fn new() -> Runtime {
        let mut interpreter = Runtime {
            stack: vec![StackFrame {
                values: vec![],
                stack: vec![],
            }],
            memory: Memory {
                // gc_graph: HashMap::new(),
                //memory: vec![],
                recently_deallocated_addr: vec![],
            },
            special_values: HashMap::new(),
            modules: HashMap::new(),
            program_consts: vec![],
            prog_counter: Cell::new(0),
            builtin_type_addrs: BuiltinTypeAddresses {
                int: std::ptr::null_mut(),
                float: std::ptr::null_mut(),
                boolean: std::ptr::null_mut(),
                string: std::ptr::null_mut(),
                list: std::ptr::null_mut(),
                true_val: std::ptr::null_mut(),
                false_val: std::ptr::null_mut(),
            },
        };

        let type_type = interpreter.allocate_and_write(PyObject {
            type_addr: std::ptr::null_mut(),
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

        interpreter.make_const(type_type);
        interpreter.make_const(none_type);
        interpreter.make_const(none_value);
        interpreter.make_const(not_implemented_type);
        interpreter.make_const(not_implemented_value);
        interpreter.make_const(callable_type);
        interpreter.make_const(module_type);

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
            .insert(SpecialValue::CallableType, callable_type);
        interpreter
            .special_values
            .insert(SpecialValue::ModuleType, module_type);

        interpreter
            .modules
            .insert(BUILTIN_MODULE.to_string(), builtin_module_obj);

        return interpreter;
    }

    pub fn store_const(&mut self, value: MemoryAddress) {
        self.program_consts.push(value);
        self.make_const(value);
    }

    pub fn get_const(&self, index: usize) -> MemoryAddress {
        return *self.program_consts.get(index).unwrap();
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

    pub fn add_to_module(&mut self, module: &str, name: &str, pyobject_addr: MemoryAddress) {
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

    pub fn get_pyobj_byaddr_mut(&mut self, addr: MemoryAddress) -> &mut PyObject {
        return self.memory.get_mut(addr);
    }

    pub fn increase_refcount(&mut self, addr: MemoryAddress) {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::Object {
            raw_data: _,
            refcount,
        } = &mut pyobj.structure
        {
            *refcount = *refcount + 1;
            //eprintln!("INCREASED addr {} from {} to {}", addr, *refcount - 1, *refcount);
        }
    }

    pub fn decrease_refcount(&mut self, addr: MemoryAddress) {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::Object {
            raw_data: _,
            refcount,
        } = &mut pyobj.structure
        {
            if *refcount > 0 {
                *refcount = *refcount - 1;
            }

            //eprintln!("Decreased addr {} from {} to {}", addr, *refcount + 1, *refcount);
            if *refcount <= 0 {
                //eprintln!("Deallocated addr {}", addr);
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

    pub fn make_const(&mut self, addr: MemoryAddress) {
        self.memory.make_const(addr);
    }

    pub fn allocate_type_byaddr_raw_struct(
        &mut self,
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
        &mut self,
        type_name: &str,
        raw_data: BuiltInTypeData,
    ) -> MemoryAddress {
        let type_addr = self.find_in_module(BUILTIN_MODULE, type_name).unwrap();
        self.allocate_type_byaddr_raw(type_addr, raw_data)
    }

    pub fn allocate_type_byaddr_raw(
        &mut self,
        type_addr: MemoryAddress,
        raw_data: BuiltInTypeData,
    ) -> MemoryAddress {
        return self.memory.allocate_and_write_builtin(type_addr, raw_data);
    }

    pub fn create_callable_pyobj(
        &mut self,
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

    pub fn register_bounded_func<F>(
        &mut self,
        module_name: &str,
        type_name: &str,
        name: &str,
        callable: F,
    ) where
        F: Fn(&mut Runtime, CallParams) -> MemoryAddress + 'static,
    {
        let pycallable = PyCallable {
            code: Box::new(callable),
        };
        let func_addr = self.create_callable_pyobj(pycallable, Some(name.to_string()));
        let module = self.find_in_module(module_name, type_name).unwrap();
        let pyobj = self.get_pyobj_byaddr_mut(module);
        if let PyObjectStructure::Type {
            name: _,
            methods,
            functions: _,
            supertype: _,
        } = &mut pyobj.structure
        {
            methods.insert(name.to_string(), func_addr);
        } else {
            panic!("Object is not a type: {}.{}", module_name, type_name);
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
                "get_raw_data_of_pyobj cannot be called on {:?}",
                pyobj.structure
            )
        }
    }

    pub fn get_raw_data_of_pyobj_mut(&mut self, addr: MemoryAddress) -> &mut BuiltInTypeData {
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

    unsafe fn very_bad_function<T>(reference: &T) -> &mut T {
        let const_ptr = reference as *const T;
        let mut_ptr = const_ptr as *mut T;
        &mut *mut_ptr
    }

    pub fn callable_call(
        &self,
        callable_addr: MemoryAddress,
        call_params: CallParams,
    ) -> MemoryAddress {
        //TODO: I know this is *bad bad very bad*, but the fields I use here (mostly native callable code) are *not* changed
        //during normal execution of the program. For now this is unsafe but *very* practical.
        //Maybe I should store the callables elsewhere, so that I start the code borrow outside of the Runtime borrow?

        let call_self = unsafe { Runtime::very_bad_function(self) };
        let callable_pyobj = self.get_pyobj_byaddr(callable_addr);

        if let PyObjectStructure::NativeCallable { code, name: _ } = &callable_pyobj.structure {
            return (code.code)(call_self, call_params);
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

    pub fn bounded_function_call_byaddr(
        &mut self,
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
                    panic!("Not a method at addr: {:p}", method_addr);
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

    pub fn unbounded_function_call_byaddr(
        &mut self,
        function_addr: MemoryAddress,
        params: &[MemoryAddress],
    ) -> MemoryAddress {
        let unbounded_func_pyobj = self.get_pyobj_byaddr(function_addr);

        let call_params = match &unbounded_func_pyobj.structure {
            PyObjectStructure::NativeCallable { code: _, name: _ } => CallParams {
                bound_pyobj: None,
                func_address: function_addr,
                func_name: None,
                params,
            },
            _ => {
                panic!("Not a function at addr: {:p}", function_addr);
            }
        };

        self.callable_call(function_addr, call_params)
    }

    pub fn new_stack_frame(&mut self) {
        self.stack.push(StackFrame {
            values: vec![],
            stack: vec![],
        })
    }

    pub fn pop_stack(&mut self) -> MemoryAddress {
        match self.stack.last_mut().unwrap().stack.pop() {
            Some(addr) => addr,
            None => panic!("Attempt to pop on empty stack!"),
        }
    }

    pub fn get_stack_offset(&self, offset: isize) -> MemoryAddress {
        let len = self.stack.last().unwrap().stack.len() as isize;
        let index = len - 1 + offset;
        return self.stack.last().unwrap().stack[index as usize];
    }

    pub fn top_stack(&self) -> MemoryAddress {
        return self.get_stack_offset(0);
    }

    pub fn push_onto_stack(&mut self, value: MemoryAddress) {
        self.stack.last_mut().unwrap().stack.push(value)
    }

    pub fn pop_stack_frame(&mut self) {
        match self.stack.pop() {
            Some(stack_frame) => {
                for addr in stack_frame.stack.iter() {
                    self.decrease_refcount(*addr)
                }
            }
            None => panic!("Attempt to pop on empty stack frames!"),
        }
    }

    pub fn bind_local(&mut self, name: usize, addr: MemoryAddress) {
        let current_frame = self.stack.last_mut().unwrap();

        //@Todo horrible stuff here
        if current_frame.values.len() == name {
            current_frame.values.push(addr);
        } else if current_frame.values.len() > name {
            current_frame.values[name] = addr
        } else if current_frame.values.len() < name {
            while current_frame.values.len() < name {
                current_frame.values.push(std::ptr::null_mut());
            }
            current_frame.values.push(addr);
        }
    }

    pub fn get_local(&mut self, name: usize) -> Option<MemoryAddress> {
        let current_frame = self.stack.last_mut().unwrap();
        current_frame.values.get(name).map(|a| *a)
    }

    pub fn allocate_and_write(&mut self, data: PyObject) -> MemoryAddress {
        self.memory.allocate_and_write(data)
    }

    pub fn get_pc(&self) -> usize {
        self.prog_counter.get()
    }

    pub fn jump_pc(&self, delta: isize) -> usize {
        self.prog_counter
            .set((self.prog_counter.get() as isize + delta) as usize);
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
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtin_types::register_builtins;

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
