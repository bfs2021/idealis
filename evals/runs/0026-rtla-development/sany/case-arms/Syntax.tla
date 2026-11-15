---- MODULE Syntax ----
VARIABLE x
Expr ==
  CASE x = 0 -> 1 [] OTHER -> 2
====