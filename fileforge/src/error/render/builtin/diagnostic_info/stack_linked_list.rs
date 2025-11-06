use core::fmt::Debug;

pub struct StackSinglyLinkedList<'l, T> {
  pub contents: T,
  pub previous: Option<&'l StackSinglyLinkedList<'l, T>>,
}

impl<'l, T: Debug> Debug for StackSinglyLinkedList<'l, T> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("StackSinglyLinkedList ").unwrap();

    let mut list = f.debug_list();

    self.iter().for_each(|el| {
      list.entry(el);
    });

    list.finish()
  }
}

impl<'l, T> StackSinglyLinkedList<'l, T> {
  pub fn iter<'a>(&'a self) -> StackSinglyLinkedListIter<'a, T> {
    StackSinglyLinkedListIter {
      current: Some(self),
    }
  }
}

pub struct StackSinglyLinkedListIter<'l, T> {
  current: Option<&'l StackSinglyLinkedList<'l, T>>,
}

impl<'l, T> Iterator for StackSinglyLinkedListIter<'l, T> {
  type Item = &'l T;

  fn next(&mut self) -> Option<Self::Item> {
    let c = self.current?;
    self.current = c.previous;
    return Some(&c.contents);
  }
}
