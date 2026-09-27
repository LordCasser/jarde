package cf06;

/* JADX INFO: loaded from: input.jar:cf06/InnerAssignCases.class */
public class InnerAssignCases {
    private String field;
    private String swapField;

    public static int lengthBranch(String str) {
        int length;
        if (str.isEmpty() || (length = str.length()) > 5) {
            return -1;
        }
        return length;
    }

    public boolean assignedAndChecked(String str) {
        String str2;
        return call(str) || ((str2 = this.field) != null && str2.isEmpty());
    }

    private boolean call(String str) {
        this.field = this.swapField;
        return str.isEmpty();
    }

    public boolean run(String str, String str2) {
        this.field = null;
        this.swapField = str2;
        return assignedAndChecked(str);
    }
}
