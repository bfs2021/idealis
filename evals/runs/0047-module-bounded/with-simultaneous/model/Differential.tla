---- MODULE Differential ----
CONSTANTS A, B
VARIABLE x
I == INSTANCE Pair WITH A <- B, B <- A
Init == x = I!Value
Next == UNCHANGED x
Safe == x = <<2,1>>
====