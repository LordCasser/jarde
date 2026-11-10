public final class CapturedTypeDriver {
    public static void main(String[] args) {
        String whenTrue = CapturedType.run(true);
        String whenFalse = CapturedType.run(false);
        if (!"number".equals(whenTrue)) throw new AssertionError("run(true) mismatch");
        if (!"number".equals(whenFalse)) throw new AssertionError("run(false) mismatch");
        System.out.println("PASS: run(true)=number; run(false)=number");
    }
}
