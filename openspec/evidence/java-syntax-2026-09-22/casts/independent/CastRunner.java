import java.util.concurrent.Callable;
public class CastRunner {
    static void run(String name, Callable<Object> f) {
        try { System.out.println(name + "=" + f.call()); }
        catch (Throwable t) { System.out.println(name + "=" + t.getClass().getName()); }
    }
    public static void main(String[] args) {
        CastAudit.value = "ok";
        run("direct", () -> CastAudit.direct("text"));
        run("nullCast", () -> CastAudit.direct(null));
        run("badCast", () -> CastAudit.direct(new Object()));
        run("receiver", () -> CastAudit.receiver("text"));
        run("nullReceiver", () -> CastAudit.receiver(null));
        run("array", () -> CastAudit.array(new int[]{4, 9}, 1));
        run("arrayType", () -> CastAudit.array(new long[]{4, 9}, 1));
        run("arrayNull", () -> CastAudit.array(null, 0));
        run("multi", () -> CastAudit.multi(new String[][]{{"a"}})[0][0]);
        run("multiType", () -> CastAudit.multi(new Object[1][1]));
        run("nestedNull", () -> CastAudit.nested(null));
        run("nestedIntermediate", () -> CastAudit.nested(Integer.valueOf(3)));
        run("staticField", () -> CastAudit.staticField());
        run("instanceField", () -> new CastAudit().instanceField());
        run("local", () -> CastAudit.local("kept"));
        CastAudit.calls = 0;
        run("callCast", () -> CastAudit.callCast());
        System.out.println("callsAfterSuccess=" + CastAudit.calls);
        CastAudit.value = Integer.valueOf(2);
        run("callCastFailure", () -> CastAudit.callCast());
        System.out.println("callsAfterFailure=" + CastAudit.calls);
        run("unusedFailure", () -> CastAudit.unusedLocal(new Object()));
        System.out.println("callsAfterUnusedFailure=" + CastAudit.calls);
        run("unusedSuccess", () -> CastAudit.unusedLocal("valid"));
        System.out.println("callsAfterUnusedSuccess=" + CastAudit.calls);
    }
}
