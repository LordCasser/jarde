public class IntegerSwitchAuditRunner {
    public static void main(String[] args) {
        int[] inputs = {-1, 1, 2, 7, 10, 20, 30, 4, 8, 9, 2748};
        for (int x : inputs) {
            System.out.println(x + "=" + IntegerSwitchAudit.grouped(x) + ","
                    + IntegerSwitchAudit.fallthrough(x) + ","
                    + IntegerSwitchAudit.noDefault(x) + ","
                    + IntegerSwitchAudit.labelConstant(x));
        }
    }
}
