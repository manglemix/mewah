use std::{alloc::Layout, marker::PhantomData, ptr::Unique};

use serde::{Deserialize, Serialize};

use crate::value::AnyValue;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FieldType {
    Int { initial: isize },
    Float { initial: f32 },
    String { initial: Box<str> },
    Any { initial: AnyValue }
}

impl FieldType {
    pub(crate) unsafe fn initialize_field(&self, field: *mut u8) {
        match self {
            FieldType::Int { initial } => {
                let field: *mut isize = field.cast();
                field.write(*initial);
            }
            FieldType::Float { initial } => {
                let field: *mut f32 = field.cast();
                field.write(*initial);
            }
            FieldType::String { initial } => {
                let field: *mut String = field.cast();
                field.write(initial.to_string());
            }
            FieldType::Any { initial } => {
                let field: *mut AnyValue = field.cast();
                field.write(initial.clone());
            }
        }
    }

    #[inline(always)]
    pub(crate) fn align(&self) -> usize {
        match self {
            FieldType::Int { .. } => Layout::new::<isize>().align(),
            FieldType::Float { .. } => Layout::new::<f32>().align(),
            FieldType::String { .. } => Layout::new::<String>().align(),
            FieldType::Any { .. } => Layout::new::<AnyValue>().align(),
        }
    }

    #[inline(always)]
    pub(crate) fn size(&self) -> usize {
        match self {
            FieldType::Int { .. } => Layout::new::<isize>().size(),
            FieldType::Float { .. } => Layout::new::<f32>().size(),
            FieldType::String { .. } => Layout::new::<String>().size(),
            FieldType::Any { .. } => Layout::new::<AnyValue>().size(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ComponentType {
    layout_size: usize,
    layout_align: usize,
    fields: Box<[FieldType]>,
}

struct ComponentMetadata {
    is_alive: bool,
}

impl Default for ComponentMetadata {
    fn default() -> Self {
        Self { is_alive: true }
    }
}

struct ComponentVec {
    components_buf: Unique<[u8]>,
    components_metadata: Vec<ComponentMetadata>,
    component_count: usize,
    layout: Layout,
    component_type: ComponentType,
    array_offset: usize,
}

const COMPONENTS_VEC_GROWTH_RATE: usize = 2;

#[derive(Clone, Copy)]
pub(crate) struct ComponentRef<'a> {
    ptr: *const [u8],
    _phantom: PhantomData<&'a ()>,
}

impl<'a> ComponentRef<'a> {
    /// Returns a pointer to a byte slice containing the whole component
    /// 
    /// # Safety
    /// The padding bytes of the component are uninitialized, so it is NOT
    /// safe to dereference the whole slice at once
    pub fn as_ptr(&self) -> *const [u8] {
        self.ptr
    }
}

unsafe impl<'a> Send for ComponentRef<'a> {}
unsafe impl<'a> Sync for ComponentRef<'a> {}


#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ComponentIndex(usize);


impl ComponentVec {
    fn make_component(&mut self) -> usize {
        let new_index = self.component_count;
        self.component_count += 1;

        if new_index >= self.capacity() {
            let new_len = self.bytes_capacity() * COMPONENTS_VEC_GROWTH_RATE;
            let arr_layout = self.layout.repeat_packed(new_len).expect("Out of memory");

            unsafe {
                let ptr = std::alloc::alloc(arr_layout);
                ptr.copy_from_nonoverlapping(
                    self.components_buf.as_ptr().as_mut_ptr(),
                    self.bytes_capacity(),
                );
                let ptr = std::ptr::slice_from_raw_parts_mut(ptr, new_len);
                self.components_buf = Unique::new_unchecked(ptr);
            }
        }

        self.components_metadata.push(Default::default());

        let ptr = unsafe {
            let ptr = self
                .components_buf
                .as_ptr()
                .get_unchecked_mut(new_index * self.array_offset);
            std::ptr::slice_from_raw_parts_mut(ptr, self.layout.size())
        };

        let mut index = 0usize;
        for field_type in self.component_type.fields.iter() {
            unsafe {
                field_type.initialize_field(ptr.get_unchecked_mut(index));
            }
            index += field_type.size();
            index = index.next_multiple_of(field_type.align());
        }

        new_index
    }

    fn get_component(&self, index: usize) -> Option<ComponentRef> {
        let metadata = self.components_metadata.get(index)?;
        if !metadata.is_alive {
            return None;
        }
        Some(ComponentRef {
            ptr: unsafe {
                std::ptr::slice_from_raw_parts(
                    self.components_buf.as_ptr().get_unchecked_mut(index),
                    self.layout.size(),
                )
            },
            _phantom: PhantomData,
        })
    }

    #[inline(always)]
    fn bytes_capacity(&self) -> usize {
        self.components_buf.as_ptr().len()
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.bytes_capacity() / self.layout.size()
    }
}

impl From<ComponentType> for ComponentVec {
    fn from(value: ComponentType) -> Self {
        let layout = Layout::from_size_align(value.layout_size, value.layout_align)
            .expect("Component Layout should be valid");
        Self {
            components_buf: unsafe { Unique::new_unchecked(Box::into_raw(Box::new([]))) },
            component_type: value,
            component_count: 0,
            array_offset: layout.size() + layout.padding_needed_for(layout.align()),
            layout,
            components_metadata: Vec::new(),
        }
    }
}
