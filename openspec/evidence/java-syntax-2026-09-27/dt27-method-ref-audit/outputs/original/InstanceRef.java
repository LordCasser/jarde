package dt27;

import java.util.function.IntSupplier;

class InstanceRef {
    private final int value;

    InstanceRef(int value) {
        this.value = value;
    }

    int number() {
        return value;
    }

    IntSupplier supplier() {
        return this::number;
    }
}
