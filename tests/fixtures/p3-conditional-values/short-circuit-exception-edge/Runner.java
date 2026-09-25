public final class Runner {
    public static void main(String[] args) {
        ExceptionShortCircuit.result = false;
        ExceptionShortCircuit.calls = 0;
        try {
            System.out.println("left=false,returned=" + ExceptionShortCircuit.assign(false) + ",field=" + ExceptionShortCircuit.result + ",calls=" + ExceptionShortCircuit.calls);
            ExceptionShortCircuit.result = true;
            ExceptionShortCircuit.calls = 0;
            System.out.println("left=true,returned=" + ExceptionShortCircuit.assign(true) + ",field=" + ExceptionShortCircuit.result + ",calls=" + ExceptionShortCircuit.calls);
        } catch (Throwable t) {
            throw new AssertionError(t);
        }
    }
}
