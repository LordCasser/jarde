public class PlainFinallyCopyDriver {
    static RuntimeException exception(String name, Throwable cause, Throwable one, Throwable two) {
        RuntimeException value = new RuntimeException(name, cause);
        value.addSuppressed(one); value.addSuppressed(two); return value;
    }
    static void check(String name, int[] values, int throwAt, boolean closeFails, boolean openFails, int answer, String trace) throws Exception {
        Throwable cause = new RuntimeException("cause"), one = new RuntimeException("one"), two = new RuntimeException("two");
        RuntimeException read = exception("read", cause, one, two);
        RuntimeException close = exception("close", cause, two, one);
        RuntimeException open = exception("open", cause, one, two);
        ContractResource.values = values; ContractResource.next = 0; ContractResource.throwAt = throwAt;
        ContractResource.events = new StringBuilder(); ContractResource.readFailure = read;
        ContractResource.closeFailure = closeFails ? close : null; ContractResource.openFailure = openFails ? open : null;
        Throwable actual = null; int result = 777;
        try { result = PlainFinallyCopy.run("input"); } catch (Throwable failure) { actual = failure; }
        Throwable expected = openFails ? open : closeFails ? close : throwAt >= 0 ? read : null;
        int expectedReads = trace.split("read", -1).length - 1;
        if (ContractResource.next != expectedReads || actual != expected || result != (expected == null ? answer : 777) || !trace.equals(ContractResource.events.toString())) throw new AssertionError(name + ": result/identity/trace " + result + "/" + actual + "/" + ContractResource.events);
        for (RuntimeException failure : new RuntimeException[]{read, close, open}) {
            Throwable[] suppressed = failure.getSuppressed();
            if (failure.getCause() != cause || suppressed.length != 2 || suppressed[0] != (failure == close ? two : one) || suppressed[1] != (failure == close ? one : two)) throw new AssertionError(name + ": cause or suppression changed");
        }
        System.out.println(name + ":result=" + result + ";throw=" + (actual == null ? "none" : actual == open ? "open" : actual == close ? "close" : "read") + ";next=" + ContractResource.next + ";events=" + trace + ";identity/cause/suppression=OK");
    }
    public static void main(String[] args) throws Exception {
        check("empty", new int[]{-1,-1}, -1,false,false,-1,"open;read0;read1;close;");
        check("zero", new int[]{0,-1}, -1,false,false,0,"open;read0;read1;close;");
        check("positive", new int[]{65,66,-1}, -1,false,false,132,"open;read0;read1;read2;close;");
        check("negative", new int[]{-2,3,-1}, -1,false,false,1,"open;read0;read1;read2;close;");
        check("first-throw", new int[]{}, 0,false,false,777,"open;read0;close;");
        check("later-throw", new int[]{65,66}, 2,false,false,777,"open;read0;read1;read2;close;");
        check("normal-close-throw", new int[]{65,66,-1}, -1,true,false,777,"open;read0;read1;read2;close;");
        check("both-throw", new int[]{65}, 1,true,false,777,"open;read0;read1;close;");
        check("constructor-throw", new int[]{}, -1,true,true,777,"open;");
    }
}
