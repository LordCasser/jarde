package defpackage;

import java.io.PrintStream;
import java.util.Objects;

/* JADX INFO: loaded from: fixture.jar:OuterReceiverCases.class */
public class OuterReceiverCases extends ReceiverBase {
    private final int state;

    OuterReceiverCases(int state) {
        this.state = state;
    }

    /* JADX INFO: Access modifiers changed from: package-private */
    @Override // defpackage.ReceiverBase
    public int value() {
        return 2;
    }

    /* JADX INFO: loaded from: fixture.jar:OuterReceiverCases$Member.class */
    class Member extends ReceiverMemberBase {
        Member() {
        }

        String compare(OuterReceiverCases other) {
            return other.state + ":" + OuterReceiverCases.this.state + ":" + OuterReceiverCases.super.value() + ":" + value();
        }
    }

    /* JADX INFO: loaded from: fixture.jar:OuterReceiverCases$StaticNested.class */
    static class StaticNested {
        StaticNested() {
        }

        int read(OuterReceiverCases other) {
            return other.state;
        }
    }

    public static void main(String[] args) {
        OuterReceiverCases outer = new OuterReceiverCases(10);
        OuterReceiverCases other = new OuterReceiverCases(20);
        PrintStream printStream = System.out;
        Objects.requireNonNull(outer);
        printStream.println(outer.new Member().compare(other));
    }
}
