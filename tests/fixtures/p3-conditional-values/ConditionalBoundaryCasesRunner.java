public class ConditionalBoundaryCasesRunner {
    public static void main(String[] args) {
        System.out.println("loop=" + ConditionalBoundaryCases.loop(true, 2));
        ConditionalBoundaryCases.trace = 0;
        System.out.println("guarded:true=" + ConditionalBoundaryCases.guarded(true)
                + ":trace=" + ConditionalBoundaryCases.trace);
        ConditionalBoundaryCases.trace = 0;
        System.out.println("guarded:false=" + ConditionalBoundaryCases.guarded(false)
                + ":trace=" + ConditionalBoundaryCases.trace);
        ConditionalBoundaryCases.fail = true;
        ConditionalBoundaryCases.trace = 0;
        System.out.println("guarded:throw=true=" + ConditionalBoundaryCases.guarded(true)
                + ":trace=" + ConditionalBoundaryCases.trace);
        ConditionalBoundaryCases.trace = 0;
        System.out.println("guarded:throw=false=" + ConditionalBoundaryCases.guarded(false)
                + ":trace=" + ConditionalBoundaryCases.trace);
        System.out.println("unknown:true=" + ConditionalBoundaryCases.unknownType(true));
        System.out.println("unknown:false=" + ConditionalBoundaryCases.unknownType(false));
    }
}
