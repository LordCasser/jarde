// @method nestedEffects(ILjava/lang/StringBuilder;)I
// @declaration a static method of `NestedConditional`, member flags 0x0009
// recovered from bytecode; presentation is not claimed to compile
{
    if (arg0 > outerLimit(arg0, arg1)) {
        if (arg0 > innerLimit(arg0, arg1)) {
            // @bytecode 18 23
            // the saved producer at BCI 23 has no bounded final expression consumer
        } else {
            // @bytecode 29 33 34
            // the saved producer at BCI 34 has no bounded final expression consumer
        }
    } else {
        // @bytecode 40 45
        // the saved producer at BCI 45 has no bounded final expression consumer
    }
    // @bytecode 48
    // the value at BCI 48 is the entry state of stack depth 0, which no instruction produced
    // @bytecode 49 50
    // the statement at BCI 50 reads `local2`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
}
