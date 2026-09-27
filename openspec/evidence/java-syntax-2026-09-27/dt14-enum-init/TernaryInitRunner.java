package dt14;

public final class TernaryInitRunner {
    public static void main(String[] args) {
        System.out.println("ternary=" + TernaryInit.FIRST.code() + ":"
                + TernaryInit.SECOND.code() + ":" + TernaryInit.ANY.code() + ":"
                + TernaryInit.calls);
    }
}
