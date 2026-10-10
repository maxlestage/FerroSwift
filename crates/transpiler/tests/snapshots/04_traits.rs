use std::fmt;

pub trait Describe {
    fn name(&self) -> String;

    fn describe(&self) -> String {
        format!("This is {}", self.name())
    }
}

pub struct Dog {
    pub age: u8,
}

impl Describe for Dog {
    fn name(&self) -> String {
        format!("a dog of {}", self.age)
    }
}

impl fmt::Display for Dog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Dog({})", self.age)
    }
}

pub fn loudest<T: Describe + fmt::Display>(items: &[T]) -> Option<String> {
    items.iter().map(|i| i.to_string()).max()
}

pub struct Stack<T> {
    items: Vec<T>,
}

impl<T: Clone> Stack<T> {
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    pub fn peek(&self) -> Option<T> {
        self.items.last().cloned()
    }
}
