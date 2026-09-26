package defpackage;

/* JADX INFO: loaded from: fixture.jar:OuterLocalBoundary.class */
class OuterLocalBoundary extends ReceiverBase {
    private final int state;

    OuterLocalBoundary(int state) {
        this.state = state;
    }

    /* JADX WARN: Type inference failed for: r0v0, types: [OuterLocalBoundary$1Local] */
    String compareWithLocal(final OuterLocalBoundary other) {
        return new Object(this) { // from class: OuterLocalBoundary.1Local
            final /* synthetic */ OuterLocalBoundary this$0;

            {
                this.this$0 = this;
            }

            String read() {
                return other.state + ":" + this.this$0.state + ":" + OuterLocalBoundary.super.value();
            }
        }.read();
    }

    public static void main(String[] args) {
        OuterLocalBoundary outer = new OuterLocalBoundary(10);
        OuterLocalBoundary other = new OuterLocalBoundary(20);
        System.out.println(outer.compareWithLocal(other));
    }
}
