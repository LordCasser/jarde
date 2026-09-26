class StaticNestedIllegal extends ReceiverBase {
    static class Nested {
        int invalid() {
            return StaticNestedIllegal.this.value() + StaticNestedIllegal.super.value();
        }
    }
}
