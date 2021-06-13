use crate::bytecode::program::*;
use crate::runtime::datamodel::*;
use crate::runtime::memory::*;
use std::cell::Cell;
use std::collections::HashMap;

/* this is done by somewhat following the python data model in https://docs.python.org/3/reference/datamodel.html */

pub struct PyCallable {
    pub code: Box<dyn Fn(&VM, CallParams) -> MemoryAddress>,
}

impl std::fmt::Debug for PyCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[rust native code]")
    }
}

#[derive(Clone)]
pub struct PositionalParameters {
    pub params: Vec<MemoryAddress>,
}

impl<'a> Into<PositionalParameters> for &'a[MemoryAddress] {
    
    fn into(self) -> PositionalParameters { 
        PositionalParameters::from_stack_popped(self.to_vec())    
    }
}

impl PositionalParameters {
    pub fn from_stack_popped(params: Vec<MemoryAddress>) -> PositionalParameters {
        let mut reversed = params;
        reversed.reverse();
        PositionalParameters {
            params: reversed
        }
    }

    pub fn empty() -> PositionalParameters {
        PositionalParameters {
            params: vec![]
        }
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn single(addr: MemoryAddress) -> PositionalParameters {
        PositionalParameters {
            params: vec![addr]
        }
    }
}

pub struct CallParams<'a> {
    pub func_address: MemoryAddress,
    pub func_name: Option<&'a str>,
    pub params: PositionalParameters,
}

pub struct FunctionCallParams {
    pub params: Vec<MemoryAddress>,
}


pub struct MethodCallParams {
    pub bound_pyobj: MemoryAddress,
    pub params: Vec<MemoryAddress>,
}

impl<'a> CallParams<'a> {
    pub fn as_method(&self) -> MethodCallParams {
        let bound = self.params.params[0];
        let rest: Vec<MemoryAddress> = self.params.params.iter().skip(1).map(|x| *x).collect();
        MethodCallParams {
            bound_pyobj: bound,
            params: rest
        }
    }

    pub fn as_function(&self) -> FunctionCallParams {
        let rest: Vec<MemoryAddress> = self.params.params.iter().map(|x| *x).collect();
        FunctionCallParams {
            params: rest
        }
    }
}

use std::cell::RefCell;


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

#[derive(Debug)]
pub struct StackFrame {
    pub function_name: String,
    pub local_namespace: Vec<MemoryAddress>, //the compiler knows which index will be loaded at compile time, so no need for a HashMap here.
    pub stack: Vec<MemoryAddress>,
    pub exception: Option<MemoryAddress>,
    pub prog_counter: Cell<usize>,
}

pub struct VM {
    pub stack: RefCell<Vec<StackFrame>>,
    pub memory: UnsafeMemory,
    pub builtin_type_addrs: BuiltinTypeAddresses,
    pub special_values: HashMap<SpecialValue, MemoryAddress>,
    pub modules: HashMap<String, MemoryAddress>,
    //pub builtin_names: Vec<MemoryAddress>,
}

impl VM {
    pub fn new() -> VM {
        let memory = UnsafeMemory::new();
        let nullptr = memory.null_ptr();
        let mut interpreter = VM {
            stack: RefCell::new(vec![StackFrame {
                function_name: "__main__".to_owned(),
                local_namespace: vec![],
                stack: vec![],
                exception: None,
                prog_counter: Cell::new(0),
            }]),
            memory: memory,
            special_values: HashMap::new(),
            modules: HashMap::new(),
            //builtin_names: vec![],
            builtin_type_addrs: BuiltinTypeAddresses {
                int: nullptr,
                float: nullptr,
                boolean: nullptr,
                string: nullptr,
                list: nullptr,
                true_val: nullptr,
                false_val: nullptr,
                index_err: nullptr,
                code_object: nullptr,
            },
        };
        let type_type = interpreter.allocate_and_write(PyObject {
            type_addr: nullptr,
            properties: HashMap::new(),
            structure: PyObjectStructure::Type {
                name: String::from("type"),
                functions: HashMap::new(),
                supertype: None,
            },
            is_const: false,
        });

        interpreter.make_const(type_type);
        interpreter.special_values.insert(SpecialValue::Type, type_type);

        let module_type = interpreter.allocate_and_write(PyObject {
            type_addr: type_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Type {
                name: String::from("module"),
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
                global_namespace: HashMap::new(),
            },
            is_const: false,
        });

        let main_module_obj = interpreter.allocate_and_write(PyObject {
            type_addr: module_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::Module {
                name: MAIN_MODULE.to_string(),
                global_namespace: HashMap::new(),
            },
            is_const: false,
        });

       
        interpreter
            .modules
            .insert(BUILTIN_MODULE.to_string(), builtin_module_obj);

        interpreter
            .modules
            .insert(MAIN_MODULE.to_string(), main_module_obj);

        
        let none_type = interpreter.create_type(BUILTIN_MODULE, "None", None);


        let none_value = interpreter.allocate_and_write(PyObject {
            type_addr: none_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::None,
            is_const: false,
        });

        let not_implemented_type = interpreter.create_type(BUILTIN_MODULE, "NotImplemented", None);

        let not_implemented_value = interpreter.allocate_and_write(PyObject {
            type_addr: not_implemented_type,
            properties: HashMap::new(),
            structure: PyObjectStructure::NotImplemented,
            is_const: false,
        });

        let stop_iteration_type = interpreter.create_type(BUILTIN_MODULE, "StopIteration", Some(type_type));

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
                functions: HashMap::new(),
                supertype: None,
            },
            is_const: false,
        });


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
        &self,
        module: &str,
        name: &str,
        supertype: Option<MemoryAddress>,
    ) -> MemoryAddress {
        let created_type = PyObject {
            properties: HashMap::new(),
            type_addr: self.special_values[&SpecialValue::Type],
            structure: PyObjectStructure::Type {
                name: name.to_string(),
                functions: HashMap::new(),
                supertype,
            },
            is_const: false,
        };
        let type_address = self.allocate_and_write(created_type);
        let module_addr = *self.modules.get(module).unwrap();
        let pyobj = self.get_pyobj_byaddr_mut(module_addr);
        match &mut pyobj.structure {
            PyObjectStructure::Module {
                global_namespace, ..
            } => match global_namespace.get(name) {
                Some(_) => {
                    panic!("Name already exists in module {}: {}", module, name);
                }
                None => {
                    global_namespace.insert(name.to_string(), type_address);
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
            PyObjectStructure::Module {
                global_namespace, ..
            } => match global_namespace.get(name) {
                Some(_) => {
                    panic!("Name already exists in module {}: {}", module, name);
                }
                None => {
                    global_namespace.insert(name.to_string(), pyobject_addr);
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
            PyObjectStructure::Module {
                global_namespace, ..
            } => global_namespace.get(name).map(|addr| *addr),
            _ => panic!("Tried to find name {:?} in module, but Object is not module: {:?}",name, module_pyobj.structure),
        }
    }

    pub fn get_obj_property(&self, addr: MemoryAddress, attr_name: &str) -> Option<MemoryAddress> {
        let pyobj = self.get_pyobj_byaddr(addr);
        return pyobj.properties.get(attr_name).map(|x| *x);
    }

    pub fn get_pyobj_byaddr(&self, addr: MemoryAddress) -> &PyObject {
        return self.memory.get(addr);
    }

    pub fn get_pyobj_byaddr_mut(&self, addr: MemoryAddress) -> &mut PyObject {
        return self.memory.get_mut(addr);
    }

    pub fn get_refcount(&self, addr: MemoryAddress) -> i32 {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::Object { refcount, .. } = &mut pyobj.structure {
            return (*refcount) as i32;
        } else {
            return -1;
        }
    }

    pub fn get_function_name(&self, addr: MemoryAddress) -> String {
        let pyobj = self.get_pyobj_byaddr_mut(addr);
        if let PyObjectStructure::NativeCallable { name, .. } = &mut pyobj.structure {
            if let Some(n) = name {
                return n.clone();
            } else {
                return "unknown".to_owned();
            }
        } else if let PyObjectStructure::UserDefinedFunction { qualname, .. } = &mut pyobj.structure {
            return qualname.to_owned();
        } else if let PyObjectStructure::Type {name, ..} = &mut pyobj.structure {
            return name.clone();
        } else {
            return "unknown".to_owned()
        }
    }

    pub fn set_attribute(&self, obj: MemoryAddress, attr: String, value: MemoryAddress) {
        let pyobj = self.get_pyobj_byaddr_mut(obj);
        pyobj.properties.insert(attr, value);
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
        if let PyObjectStructure::Object { refcount, .. } = &mut pyobj.structure {
            if *refcount > 0 { //to prevent overflow, maybe this method shouldnt be called when intending to delete object
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
                functions, supertype, ..
            } => match functions.get(method_name) {
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
            PyObjectStructure::Type { name, .. } => {
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
        qualname: String,
        defaults: Vec<MemoryAddress>
    ) -> MemoryAddress {
        let obj = PyObject {
            properties: HashMap::new(),
            type_addr: self.builtin_type_addrs.code_object,
            structure: PyObjectStructure::UserDefinedFunction { code, qualname, defaults },
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

    pub fn create_unbounded_callable_pyobj(
        &self,
        callable: PyCallable,
        name: Option<String>
    ) -> MemoryAddress {
        self.allocate_type_byaddr_raw_struct(
            self.special_values[&SpecialValue::CallableType],
            PyObjectStructure::NativeCallable {
                code: callable,
                name,
                is_bound: false
            },
        )
    }

    pub fn create_bounded_callable_pyobj(
        &self,
        callable: PyCallable,
        name: Option<String>
    ) -> MemoryAddress {
        self.allocate_type_byaddr_raw_struct(
            self.special_values[&SpecialValue::CallableType],
            PyObjectStructure::NativeCallable {
                code: callable,
                name,
                is_bound: true
            },
        )
    }

    pub fn register_type_unbounded_func<F>(
        &self,
        type_addr: MemoryAddress,
        name: &str,
        callable: F,
    ) where
        F: Fn(&VM, CallParams) -> MemoryAddress + 'static,
    {
        let pycallable = PyCallable {
            code: Box::new(callable),
        };
        let func_addr = self.create_unbounded_callable_pyobj(pycallable, Some(name.to_string()));
        let pyobj_type = self.get_pyobj_byaddr_mut(type_addr);
        if let PyObjectStructure::Type { functions, .. } = &mut pyobj_type.structure {
            functions.insert(name.to_string(), func_addr);
        } else {
            panic!("Object is not a type: {:?}", pyobj_type.structure);
        }
    }

    pub fn register_bounded_func<F>(
        &mut self,
        module_name: &str,
        type_name: &str,
        name: &str,
        callable: F,
    ) where
        F: Fn(&VM, CallParams) -> MemoryAddress + 'static,
    {
        let type_addr = self.find_in_module(module_name, type_name).unwrap();
        self.register_bounded_func_on_addr(type_addr, name, callable);
    }

    pub fn register_bounded_func_on_addr<F>(
        &self,
        type_addr: MemoryAddress,
        name: &str,
        callable: F,
    ) where
        F: Fn(&VM, CallParams) -> MemoryAddress + 'static,
    {
        let pycallable = PyCallable {
            code: Box::new(callable),
        };
        let func_addr = self.create_bounded_callable_pyobj(pycallable, Some(name.to_string()));
        self.register_method_addr_on_type(type_addr, name, func_addr)
    }

    pub fn register_method_addr_on_type(
        &self,
        type_addr: MemoryAddress,
        name: &str,
        callable_addr: MemoryAddress,
    ) {
        let pyobj_type = self.get_pyobj_byaddr_mut(type_addr);
        if let PyObjectStructure::Type { functions, .. } = &mut pyobj_type.structure {
            functions.insert(name.to_string(), callable_addr);
        } else {
            panic!("Object is not a type: {:?}", pyobj_type);
        }
    }


    pub fn get_raw_data_of_pyobj(&self, addr: MemoryAddress) -> &BuiltInTypeData {
        let pyobj = self.get_pyobj_byaddr(addr);
        if let PyObjectStructure::Object { raw_data, .. } = &pyobj.structure {
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
        if let PyObjectStructure::UserDefinedFunction { code, .. } = &pyobj.structure {
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
        if let PyObjectStructure::Object { raw_data, .. } = &mut pyobj.structure {
            return raw_data;
        } else {
            panic!("get_raw_data_of_pyobj_mut cannot be called non-object")
        }
    }


    pub fn try_load_function(&self, addr: MemoryAddress) -> &PyObject {
        return self.get_pyobj_byaddr(self.try_load_function_addr(addr));
    }

    pub fn try_load_function_addr(&self, addr: MemoryAddress) -> MemoryAddress {
        let obj = self.get_pyobj_byaddr(addr);
        match &obj.structure {
            PyObjectStructure::NativeCallable { .. } => addr,
            PyObjectStructure::UserDefinedFunction{ .. } => addr,
            PyObjectStructure::BoundMethod {..} => addr,
            PyObjectStructure::Type { name, functions, .. } => {
                let new = functions
                    .get("__new__")
                    .expect(format!("Type {} has no __new__ function", name).as_str());
                return *new;
            }
            _ => panic!("not callable: {:?} {:?}", unsafe {&*addr}, self.stack.borrow()),
        }
    }
    

    pub fn run_function(&self, mut positional_params: PositionalParameters, 
        function_addr: MemoryAddress, bound_addr: Option<MemoryAddress>) -> (MemoryAddress, StackFrame) {
        let func_name = self.get_function_name(function_addr);
        let pyobj_func = self.try_load_function(function_addr);
        //println!("Calling function {:?}", func_name);
        match &pyobj_func.structure {
            PyObjectStructure::NativeCallable { code, name, is_bound } => {
                if *is_bound {
                    let bounded = match bound_addr {
                        Some(x) => x,
                        None => self.pop_stack()
                    };
                    //println!("Bounded value: {:?}", unsafe { &*bounded});
                    positional_params.params.insert(0, bounded);
                }
                
                self.new_stack_frame(func_name);
                let call_params = CallParams {
                    func_address: function_addr,
                    func_name: name.as_deref(),//.map(|x| x.as_str()),
                    params: positional_params,
                };
                
                let result = (code.code)(self, call_params);
                self.increase_refcount(result);
                let popped_stacked_frame = self.pop_stack_frame();
                (result, popped_stacked_frame)
            }
            PyObjectStructure::UserDefinedFunction {code, qualname, defaults} => {
                let mut expected_number_args = code.code.params.len();
                if let Some(_) = bound_addr {
                    expected_number_args -= 1; //because self is already being passed
                }
                
                if defaults.len() == 0 {
                    if expected_number_args != positional_params.params.len() {
                        panic!("Function {} expects {} parameters, but {} were provided", qualname, code.code.params.len(), positional_params.params.len());
                    }
                } else {
                    let non_default_params = expected_number_args - defaults.len();
                    if non_default_params > positional_params.params.len() {
                        panic!("Function {} expects {} non-default parameters, but {} were provided", qualname, code.code.params.len(), positional_params.params.len());
                    }

                    //pass the last N parameters that are necessary
                    let to_pass = defaults.iter().skip(positional_params.params.len());

                    for mem_addr in to_pass {
                        positional_params.params.insert(0, *mem_addr);
                    }
                }
    
                self.new_stack_frame(func_name);
                if let Some(a) = bound_addr {
                    self.bind_local(0, a); 
                    for (number, addr) in positional_params.params.iter().enumerate() {
                        self.bind_local(number + 1, *addr);
                    }
                } else {
                    for (number, addr) in positional_params.params.iter().enumerate() {
                        self.bind_local(number, *addr);
                    }
                }
                
                //what a mess
                crate::runtime::interpreter::execute_code_object(self, &code);
    
                let result_addr = self.top_stack();
                self.increase_refcount(result_addr);
                let popped_stacked_frame = self.pop_stack_frame();
                (result_addr, popped_stacked_frame)
            }
            PyObjectStructure::BoundMethod {function_address, bound_address} => {
                self.run_function(positional_params, *function_address, Some(*bound_address))
            }
            _ => {
                panic!("Not a function at addr: {:?}", function_addr);
            }
        }
    }

    pub fn call_method(
        &self,
        bound_addr: MemoryAddress,
        method_name: &str,
        params: PositionalParameters,
    ) -> Option<(MemoryAddress, StackFrame)> {
        let pyobj = self.get_pyobj_byaddr(bound_addr);
        self.get_method_addr_byname(pyobj.type_addr, method_name)
            .map(move |method_addr| {
                self.run_function(params, method_addr, Some(bound_addr))
            })
    }

    pub fn raise_exception(&self, exception_value_addr: MemoryAddress) {
        let mut stack = self.stack.borrow_mut();
        let top_stack_frame = stack.last_mut().unwrap();
        top_stack_frame.exception = Some(exception_value_addr)
    }
    
    pub fn new_stack_frame(&self, function_name: String) {
        self.stack.borrow_mut().push(StackFrame {
            function_name: function_name,
            local_namespace: vec![],
            stack: vec![],
            exception: None,
            prog_counter: Cell::new(0),
        })
    }

    pub fn pop_stack(&self) -> MemoryAddress {
        match self.stack.borrow_mut().last_mut().unwrap().stack.pop() {
            Some(addr) => addr,
            None => panic!("Attempt to pop on empty stack!"),
        }
    }

    pub fn print_traceback(&self) {
        println!("Traceback: ");
        for val in self.stack.borrow().iter().rev() {
            println!("\tat {} +{}", val.function_name, val.prog_counter.get())
        }
    }

    pub fn print_stack(&self) {
        print!("Stack: [");
        for val in self.stack.borrow().last().unwrap().stack.iter().rev() {
            let pyobj = self.get_pyobj_byaddr(*val);
            if let PyObjectStructure::Object { raw_data: raw, .. } = &pyobj.structure {
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
        self.stack
            .borrow_mut()
            .last_mut()
            .unwrap()
            .stack
            .push(value)
    }

    pub fn pop_stack_frame(&self) -> StackFrame {
        let mut stack = self.stack.borrow_mut();
        match stack.pop() {
            Some(stack_frame) => {
                for addr in stack_frame.stack.iter() {
                    self.decrease_refcount(*addr)
                }
                return stack_frame;
            }
            None => panic!("Attempt to pop on empty stack frames!"),
        }
    }

    pub fn bind_local(&self, name: usize, addr: MemoryAddress) {
        //println!("Binding local: {} to {:?}", name, unsafe{&*addr});
        let mut stack = self.stack.borrow_mut();
        let current_frame = stack.last_mut().unwrap();

        //@Todo horrible stuff here
        if current_frame.local_namespace.len() == name {
            current_frame.local_namespace.push(addr);
        } else if current_frame.local_namespace.len() > name {
            current_frame.local_namespace[name] = addr
        } else if current_frame.local_namespace.len() < name {
            while current_frame.local_namespace.len() < name {
                current_frame.local_namespace.push(self.memory.null_ptr());
            }
            current_frame.local_namespace.push(addr);
        }
    }

    pub fn get_local(&self, name: usize) -> Option<MemoryAddress> {
        let stack = self.stack.borrow();
        let current_frame = stack.last().unwrap();
        current_frame.local_namespace.get(name).map(|a| *a)
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
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let pyobj_int_addr =
            interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1));
        let result_value = interpreter.get_raw_data_of_pyobj(pyobj_int_addr).take_int();
        assert_eq!(1, result_value);
    }

    #[test]
    fn simply_instantiate_bool() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let pyobj_int_addr =
            interpreter.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(1));
        let result_value = interpreter.get_raw_data_of_pyobj(pyobj_int_addr).take_int();
        assert_eq!(1, result_value);
    }

    #[test]
    fn call_int_add_int() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1));
        let number2 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(3));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", PositionalParameters::from_stack_popped(vec![number2]))
            .unwrap().0;
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_int();

        assert_eq!(result_value, 4);
    }

    #[test]
    fn call_bool_add_int() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_builtin_type_byname_raw("bool", BuiltInTypeData::Int(1));
        let number2 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(3));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", PositionalParameters::from_stack_popped(vec![number2]))
            .unwrap().0;
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_int();

        assert_eq!(result_value, 4);
    }

    #[test]
    fn call_int_add_float() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1));
        let number2 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(3.5)));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", PositionalParameters::from_stack_popped(vec![number2]))
            .unwrap().0;
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_float();

        assert_eq!(result_value, 4.5);
    }

    #[test]
    fn call_float_add_int() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(3.5)));
        let number2 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(1));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", PositionalParameters::from_stack_popped(vec![number2]))
            .unwrap().0;
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_float();

        assert_eq!(result_value, 4.5);
    }

    #[test]
    fn call_float_add_float() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(3.4)));
        let number2 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(1.1)));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__add__", PositionalParameters::from_stack_popped(vec![number2]))
            .unwrap().0;
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_float();

        assert_eq!(result_value, 4.5);
    }

    #[test]
    fn call_float_mul_float() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(2.0)));
        let number2 = interpreter
            .allocate_builtin_type_byname_raw("float", BuiltInTypeData::Float(Float(3.0)));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__mul__", PositionalParameters::from_stack_popped(vec![number2]))
            .unwrap().0;
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_float();

        assert_eq!(result_value, 6.0);
    }

    #[test]
    fn call_int_mul_int() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let number1 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(2));
        let number2 = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(3));

        //number1.__add__(number2)
        let result = interpreter
            .call_method(number1, "__mul__", PositionalParameters::from_stack_popped(vec![number2]))
            .unwrap().0;
        let result_value = interpreter.get_raw_data_of_pyobj(result).take_int();

        assert_eq!(result_value, 6);
    }

    #[test]
    fn bind_local_test() {
        let mut interpreter = VM::new();
        register_builtins(&mut interpreter);
        let number = interpreter.allocate_builtin_type_byname_raw("int", BuiltInTypeData::Int(17));
        interpreter.bind_local(0, number);

        let addr_local = interpreter.get_local(0).unwrap();
        let result_value = interpreter.get_raw_data_of_pyobj(addr_local).take_int();

        assert_eq!(result_value, 17);
    }
}
