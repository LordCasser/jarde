package em12;

class Parent {
    String pick(Arg value) {
        return "number";
    }

    String pick(NarrowArg value) {
        return "integer";
    }
}
