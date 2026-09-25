public final class Runner {
    public static void main(String[] args) {
        for (Object x : new Object[] { null, "s", Integer.valueOf(4), new Object() }) {
            InstanceOfMerge.calls = 0;
            System.out.println(InstanceOfMerge.inverted(x) + ":" + InstanceOfMerge.calls);
            InstanceOfMerge.calls = 0;
            System.out.println(InstanceOfMerge.direct(x) + ":" + InstanceOfMerge.calls);
        }
    }
}
