package defpackage;

import java.util.ArrayList;
import java.util.List;

/* JADX INFO: loaded from: lambda-body-inline.jar:LambdaBodyInline.class */
public final class LambdaBodyInline {
    static final List<String> events = new ArrayList();
    static int captureCalls;
    static int bodyCalls;

    static int capture() {
        captureCalls++;
        events.add("capture");
        return 10;
    }

    static IntAction build(int captured) {
        return value -> {
            events.add("start:" + captured + ":" + value);
            bodyCalls++;
            events.add("end:" + captured + ":" + value);
            return captured + value;
        };
    }

    static void require(boolean condition, String message) {
        if (!condition) {
            throw new AssertionError(message);
        }
    }

    public static void main(String[] args) {
        IntAction action = build(capture());
        require(captureCalls == 1, "capture must run once during creation");
        require(bodyCalls == 0, "lambda body must not run during creation");
        require(events.toString().equals("[capture]"), "creation events: " + events);
        System.out.println("created captureCalls=" + captureCalls + " bodyCalls=" + bodyCalls + " events=" + events);
        int first = action.apply(2);
        require(first == 12, "first result: " + first);
        require(bodyCalls == 1, "first invocation count: " + bodyCalls);
        require(events.toString().equals("[capture, start:10:2, end:10:2]"), "first events: " + events);
        System.out.println("first result=" + first + " bodyCalls=" + bodyCalls + " events=" + events);
        int second = action.apply(3);
        require(second == 13, "second result: " + second);
        require(bodyCalls == 2, "second invocation count: " + bodyCalls);
        require(events.toString().equals("[capture, start:10:2, end:10:2, start:10:3, end:10:3]"), "second events: " + events);
        System.out.println("second result=" + second + " bodyCalls=" + bodyCalls + " events=" + events);
    }
}
