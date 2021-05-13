use crate::runtime::datamodel::*;
use std::cell::RefCell;
use std::collections::HashMap;


pub trait Memory {

    fn get(&self, address: MemoryAddress) -> &PyObject;

    fn get_mut(&self, address: MemoryAddress) -> &mut PyObject;

    fn make_const(&self, address: MemoryAddress);

    fn deallocate(&self, address: MemoryAddress);

    fn allocate_and_write(&self, data: PyObject) -> MemoryAddress;

    fn allocate_and_write_builtin(&self, type_addr: MemoryAddress, data: BuiltInTypeData) -> MemoryAddress;

    fn null_ptr(&self) -> MemoryAddress;
}


/*
pub type MemoryAddress = usize;
pub struct MemoryCell {
    pub data: Option<PyObject>,
    pub valid: bool,
    pub is_const: bool,
}

pub struct VecRefCellMemory {
    pub memory: RefCell<Vec<MemoryCell>>,
    //gc graph stores: Key = memory addresses, Values = Other adresses that point to that memory addr
    //Every memory address has an entry here
    //pub gc_graph: HashMap<AddressType, Vec<AddressType>>,
    pub recently_deallocated_indexes: RefCell<Vec<usize>>,
}

impl VecRefCellMemory {
    pub fn new() -> Self {
        Self {
            memory: RefCell::new(vec![]),
            recently_deallocated_indexes: RefCell::new(vec![])
        }
    }
}

impl Memory for VecRefCellMemory {

    fn get(&self, address: MemoryAddress) -> &PyObject {
        let mem_borrow = self.memory.borrow_mut();
        let cell = &mem_borrow[address];
        if cell.valid {
            return unsafe { 
                let const_ptr = cell.data.as_ref().unwrap() as *const PyObject;
                let mut_ptr = const_ptr as *mut PyObject;
                &*mut_ptr
            };
        } else {
            panic!("Attempt to read from non-valid memory address {}", address);
        }
    }

    fn get_mut(&self, address:  MemoryAddress) -> &mut PyObject {
        let mut mem_borrow = self.memory.borrow_mut();
        let cell = &mut mem_borrow[address];
        if cell.valid {
            return unsafe { 
                let const_ptr = cell.data.as_ref().unwrap() as *const PyObject;
                let mut_ptr = const_ptr as *mut PyObject;
                &mut *mut_ptr
            };
        } else {
            panic!("Attempt to read from non-valid memory address {}", address);
        }
    }

    fn make_const(&self, address: MemoryAddress) {
        let mut mem_borrow = self.memory.borrow_mut();
        let cell = &mut mem_borrow[address];
        cell.is_const = true;
    }

    fn deallocate(&self, address:  MemoryAddress) {
        let mut mem_borrow = self.memory.borrow_mut();
        let cell = &mut mem_borrow[address];
        if cell.is_const {
            return;
        };
        if cell.valid {
            cell.valid = false;
        } else {
            panic!("Attempt to dealloate already invalid memory at address in non-valid memory address {}", address)
        }

        self.recently_deallocated_indexes.borrow_mut().push(address);
    }

    fn allocate_and_write(&self, data: PyObject) ->  MemoryAddress {
        let mut mem_borrow = self.memory.borrow_mut();
        let dealloc = self.recently_deallocated_indexes.borrow_mut().pop();
        match dealloc {
            Some(address) => {
                let mut cell = &mut mem_borrow[address];
                debug_assert!(!cell.is_const);
                if cell.valid {
                    panic!(
                        "Attempt to allocate onto already occupied address {}",
                        address
                    )
                } else {
                    if let PyObjectStructure::Object{raw_data, refcount: _} = &mut cell.data.as_mut().unwrap().structure {
                        if let PyObjectStructure::Object{raw_data: new_data, refcount: _} = data.structure {
                            *raw_data = new_data;
                        } else {
                            cell.data = Some(data);
                        }
                    } else {
                        cell.data = Some(data);
                    }
                    cell.valid = true;
                }
                return address;
            }
            None => {
                mem_borrow.push(MemoryCell {
                    data: Some(data),
                    valid: true,
                    is_const: false,
                });
                return mem_borrow.len() - 1;
            }
        };
    }

    fn allocate_and_write_builtin(&self, type_addr:  MemoryAddress, data: BuiltInTypeData) ->  MemoryAddress {
        let mut mem_borrow = self.memory.borrow_mut();
        let dealloc = self.recently_deallocated_indexes.borrow_mut().pop();
        match dealloc {
            Some(address) => {
                let mut cell = &mut mem_borrow[address];
                debug_assert!(!cell.is_const);
                if cell.valid {
                    panic!(
                        "Attempt to allocate onto already occupied address {}",
                        address
                    )
                } else {
                    if let PyObjectStructure::Object{raw_data, refcount: _} = &mut cell.data.as_mut().unwrap().structure {
                        *raw_data = data;
                    } else {
                        cell.data = Some(PyObject {
                            type_addr, 
                            structure: PyObjectStructure::Object {
                                raw_data: data,
                                refcount: 1
                            },
                            is_const: false,
                            properties: HashMap::new()
                        })
                    }
                    cell.valid = true;
                }
                return address;
            }
            None => {
                mem_borrow.push(MemoryCell {
                    data: Some(PyObject {
                        type_addr, 
                        structure: PyObjectStructure::Object {
                            raw_data: data,
                            refcount: 1
                        },
                        is_const: false,
                        properties: HashMap::new()
                    }),
                    valid: true,
                    is_const: false,
                });
                return mem_borrow.len() - 1;
            }
        };
    }
    
    fn null_ptr(&self) ->  MemoryAddress { 0 }
}
*/

pub type MemoryAddress = *mut PyObject;
pub struct UnsafeMemory {
    pub recently_deallocated_addr: RefCell<Vec<*mut PyObject>>,
}

impl UnsafeMemory {
    pub fn new() -> Self {
        Self {
            recently_deallocated_addr: RefCell::new(vec![])
        }
    }

    pub fn check_mem(&self, address: MemoryAddress) {
        if self.recently_deallocated_addr.borrow().contains(&address) {
            panic!("Trying to get recently deallocated memory {:p}", address)
        }
    }

}

impl Memory for UnsafeMemory {

  
    fn get(&self, address: MemoryAddress) -> &PyObject {
        self.check_mem(address);
        if address.is_null() {
            panic!("Attempt to read from non-valid memory address null");
        }
        return unsafe { &*address };
    }

    fn get_mut(&self, address: MemoryAddress) -> &mut PyObject {
        self.check_mem(address);
        if address.is_null() {
            panic!("Attempt to read from non-valid memory address null");
        }
        return unsafe { &mut *address };
    }

    fn make_const(&self, address: MemoryAddress) {
        self.check_mem(address);
        unsafe { (*address).is_const = true };
    }

    fn deallocate(&self, address: MemoryAddress) {
        self.check_mem(address);
        if address.is_null() {
            panic!("Null pointer!");
        }
        unsafe {
            if (*address).is_const {
                return;
            }
        }
        //do not deallocate yet! Could make a new box from raw and let it go out of scope
        self.recently_deallocated_addr.borrow_mut().push(address);
    }

    fn allocate_and_write(&self, data: PyObject) -> MemoryAddress {
        let dealloc = self.recently_deallocated_addr.borrow_mut().pop();
        match dealloc {
            Some(address) => {
                unsafe {
                    println!("allocate_and_write: Writing at {:p}, data = {:?}", address, data);
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

  
    fn allocate_and_write_builtin(
        &self,
        type_addr: MemoryAddress,
        data: BuiltInTypeData,
    ) -> MemoryAddress {
        let dealloc = self.recently_deallocated_addr.borrow_mut().pop();
        match dealloc {
            Some(address) => {
                //println!("Writing builtin at {:p}, data = {:?}", address, data);
                let py_obj = unsafe { &mut *address };
                debug_assert!(!py_obj.is_const);
                
                *py_obj = PyObject {
                    type_addr,
                    structure: PyObjectStructure::Object {
                        raw_data: data,
                        refcount: 0,
                    },
                    properties: HashMap::new(),
                    is_const: false,
                };
                
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


    fn null_ptr(&self) -> MemoryAddress {std::ptr::null_mut()}

}