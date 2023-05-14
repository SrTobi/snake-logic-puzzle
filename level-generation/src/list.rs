use std::fmt;
use std::hash::Hash;
use std::iter::FromIterator;
use std::rc::Rc;

#[macro_export]
macro_rules! list {
  () => (
    $crate::list::List::nil()
  );
  ($elem:expr; $n:expr) => ({
    let list: List<_> = std::iter::FromIterator::from_iter(std::iter::repeat($elem).take($n));
    list
  });
  ($($x:expr),+ $(,)?) => ({
    let list: List<_> = std::iter::FromIterator::from_iter([$($x),+].iter().cloned());
    list
  });
}

struct ListElement<T> {
  value: T,
  next: Option<Rc<ListElement<T>>>,
}

pub struct List<T> {
  head: Option<Rc<ListElement<T>>>,
}

impl<T> List<T> {
  pub fn nil() -> Self {
    Self::from(None)
  }

  pub fn pushed(&self, value: T) -> Self {
    let elem = ListElement {
      value,
      next: self.head.clone(),
    };
    Self::from(Rc::new(elem))
  }

  pub fn push(&mut self, value: T) {
    *self = self.pushed(value);
  }

  pub fn is_empty(&self) -> bool {
    self.head.is_none()
  }

  pub fn non_empty(&self) -> bool {
    self.head.is_some()
  }

  pub fn head(&self) -> Option<&T> {
    self.head.as_ref().map(|e| &e.value)
  }

  pub fn len(&self) -> usize {
    self.iter().count()
  }

  pub fn tail(&self) -> Self {
    self
      .head
      .as_ref()
      .map_or_else(Self::nil, |elem| Self::from(elem.next.clone()))
  }

  pub fn pop(&mut self) -> Option<T>
  where
    T: Clone,
  {
    match self.head.take() {
      Some(elem_ref) => match Rc::try_unwrap(elem_ref) {
        Ok(elem) => {
          self.head = elem.next;
          Some(elem.value)
        }
        Err(ptr) => {
          self.head = ptr.next.clone();
          Some(ptr.value.clone())
        }
      },
      None => None,
    }
  }

  pub fn popped(&self) -> Option<(&T, Self)> {
    self
      .head
      .as_ref()
      .map(|elem| (&elem.value, Self::from(elem.next.clone())))
  }

  pub fn dropped(&self, len: usize) -> Self {
    let mut it = self.iter();
    let _ = it.advance_by(len);
    it.into()
  }

  pub fn extended(&self, iter: impl IntoIterator<Item = T>) -> Self {
    let mut it = iter.into_iter();
    match it.next() {
      Some(value) => {
        let start = Rc::new(ListElement { value, next: None });
        let mut cur = start.clone();

        for value in it {
          let next = Rc::new(ListElement { value, next: None });
          let cur_ref = unsafe {
            // This is ok, because cur is either the initial value or that of the last iteration,
            // and no cloning of either occurs in this function.
            Rc::get_mut_unchecked(&mut cur)
          };
          cur_ref.next = Some(next.clone());
          cur = next;
        }

        {
          let cur_ref = unsafe {
            // This is ok, because cur is either the initial value or that of the last iteration,
            // and no cloning of either occurs in this function.
            Rc::get_mut_unchecked(&mut cur)
          };
          cur_ref.next = self.head.clone();
        }

        Self::from(start)
      }
      None => Self::nil(),
    }
  }

  pub fn iter(&self) -> ListIterator<'_, T> {
    ListIterator {
      cur: self.head.as_ref(),
    }
  }
}

impl<T> Drop for List<T> {
  fn drop(&mut self) {
    let mut element = self.head.take();
    while let Some(cur) = element {
      if let Ok(mut cur) = Rc::try_unwrap(cur) {
        element = cur.next.take();
      } else {
        return;
      }
    }
  }
}

impl<T> Default for List<T> {
  fn default() -> Self {
    Self::nil()
  }
}

impl<T> Clone for List<T> {
  fn clone(&self) -> Self {
    Self::from(self.head.clone())
  }
}

impl<T: PartialEq> PartialEq for List<T> {
  fn eq(&self, other: &Self) -> bool {
    self.iter().eq(other.iter())
  }
}

impl<T: PartialEq> Eq for List<T> {}

impl<T: PartialOrd> PartialOrd for List<T> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.iter().partial_cmp(other.iter())
  }
}

impl<T: Ord> Ord for List<T> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.iter().cmp(other.iter())
  }
}

impl<T: Hash> Hash for List<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.iter().for_each(|e| e.hash(state))
  }
}

impl<T> From<Rc<ListElement<T>>> for List<T> {
  fn from(elem: Rc<ListElement<T>>) -> Self {
    Self { head: Some(elem) }
  }
}

impl<T> From<Option<Rc<ListElement<T>>>> for List<T> {
  fn from(head: Option<Rc<ListElement<T>>>) -> Self {
    Self { head }
  }
}

impl<T> FromIterator<T> for List<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    Self::nil().extended(iter)
  }
}

impl<T> Extend<T> for List<T> {
  fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
    *self = self.extended(iter);
  }
}

impl<T: fmt::Debug> fmt::Debug for List<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[")?;
    let mut it = self.iter();
    if let Some(first) = it.next() {
      write!(f, "{:?}", first)?;
    }
    for value in it {
      write!(f, ", {:?}", value)?;
    }

    write!(f, "]")
  }
}

#[derive(Clone)]
pub struct ListIterator<'l, T> {
  cur: Option<&'l Rc<ListElement<T>>>,
}

impl<'l, T> Iterator for ListIterator<'l, T> {
  type Item = &'l T;

  fn next(&mut self) -> Option<Self::Item> {
    self.cur.take().map(|old| {
      self.cur = old.next.as_ref();
      &old.value
    })
  }
}

impl<'l, T> From<ListIterator<'l, T>> for List<T> {
  fn from(it: ListIterator<'l, T>) -> Self {
    Self::from(it.cur.cloned())
  }
}

#[cfg(test)]
mod tests {
  use super::List;

  #[test]
  fn test_empty() {
    let nil: List<()> = List::nil();

    assert!(nil.is_empty());
    assert!(!nil.non_empty());

    assert_eq!(None, nil.head());
    assert_eq!(None, nil.popped());

    assert_eq!(nil, list![]);
    assert_eq!(nil.len(), 0);
  }

  #[test]
  fn test_from_n() {
    let list = list![3; 5];

    assert!(!list.is_empty());
    assert!(list.non_empty());

    assert_eq!(Some(&3), list.head());
    assert_eq!(Some((&3, list.tail())), list.popped());

    assert_eq!(vec![&3; 5], list.iter().collect::<Vec<_>>());

    assert_eq!(list.len(), 5);
  }

  #[test]
  fn test_push() {
    let mut list = list![];
    list.push(4);
    list.push(3);

    assert_eq!(list, list![3, 4]);

    list.extend([1, 2].iter().cloned());

    assert_eq!(list, list![1, 2, 3, 4]);
  }

  #[test]
  fn test_dropped() {
    assert_eq!(list![1].dropped(2), list![]);
    assert_eq!(list![1, 2].dropped(0), list![1, 2]);
    assert_eq!(list![1, 2, 3, 4].dropped(2), list![3, 4]);
  }
}
