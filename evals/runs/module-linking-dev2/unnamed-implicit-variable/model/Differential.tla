---- MODULE Differential ----
EXTENDS Naturals
VARIABLE value
INSTANCE Counter WITH Limit <- 2
Safe == value <= 2
====