class OuterLocalBoundary extends ReceiverBase {
    private final int state;

    OuterLocalBoundary(int state) { this.state = state; }

    String compareWithLocal(OuterLocalBoundary other) {
        class Local {
            String read() {
                return other.state + ":" + OuterLocalBoundary.this.state + ":"
                        + OuterLocalBoundary.super.value();
            }
        }
        return new Local().read();
    }

    public static void main(String[] args) {
        OuterLocalBoundary outer = new OuterLocalBoundary(10);
        OuterLocalBoundary other = new OuterLocalBoundary(20);
        System.out.println(outer.compareWithLocal(other));
    }
}
