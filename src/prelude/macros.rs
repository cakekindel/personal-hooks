macro_rules! m {
  ($f:item($args:tt)) => |v| $f(v, $args)
}
