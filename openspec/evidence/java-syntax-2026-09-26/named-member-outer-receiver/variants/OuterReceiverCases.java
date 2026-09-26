public class OuterReceiverCases extends ReceiverBase {
    private final int state;

    OuterReceiverCases(int state) { this.state = state; }

    @Override int value() { return 2; }

    class Member extends ReceiverMemberBase {
        String compare(OuterReceiverCases other) {
            return other.state + ":" + OuterReceiverCases.this.state + ":"
                    + OuterReceiverCases.super.value() + ":" + this.value();
        }
    }

    static class StaticNested {
        int read(OuterReceiverCases other) { return other.state; }
    }

    public static void main(String[] args) {
        OuterReceiverCases outer = new OuterReceiverCases(10);
        OuterReceiverCases other = new OuterReceiverCases(20);
        System.out.println(outer.new Member().compare(other));
    }
}

class ReceiverBase {
    int value() { return 1; }
}

class ReceiverMemberBase {
    int value() { return 3; }
}
