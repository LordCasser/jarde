package proof;

interface LeftDefault {
    default int value() {
        return 11;
    }
}

interface RightDefault {
    default int value() {
        return 22;
    }
}

interface BothDefault extends LeftDefault, RightDefault {
    default int value() {
        return 33;
    }
}
