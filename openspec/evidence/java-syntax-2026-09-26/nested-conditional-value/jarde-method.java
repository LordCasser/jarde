// @method nested(I)I
// @declaration a static method of `NestedConditional`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    if (arg0 > 10) {
        if (arg0 > 100) {
        }
    }
    // @bytecode 21
    // the value at BCI 21 is the entry state of stack depth 0, which no instruction produced
    // @bytecode 22 23
    // the statement at BCI 23 reads `local1`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
}
