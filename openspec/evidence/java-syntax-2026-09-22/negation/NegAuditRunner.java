public class NegAuditRunner {
    public static void main(String[] args) {
        for (int x : new int[]{0,1,-1,2,17,Integer.MIN_VALUE,Integer.MAX_VALUE}) {
            System.out.println(x + ":" + NegAudit.mixed(x) + ":" + NegAudit.concat(x) + ":" + NegAudit.boxed(x) + ":" + NegAudit.branch(x) + ":" + NegAudit.stored(x));
        }
        System.out.println("min=" + NegAudit.longMin());
        for (int x : new int[]{-2,0,1,4}) System.out.println("loop=" + NegAudit.loop(x));
        for (double x : new double[]{0.0,-0.0,1.5,-3.0,Double.MIN_VALUE,Double.MAX_VALUE,Double.POSITIVE_INFINITY,Double.NEGATIVE_INFINITY,Double.NaN}) {
            double value=NegAudit.floating(x, x);
            System.out.println(Double.isNaN(value)?"NaN":Long.toHexString(Double.doubleToRawLongBits(value)));
        }
    }
}
