package cf06;

public class InnerAssignCases {
    private String field;
    private String swapField;

    public static int lengthBranch(String text) {
        int length;
        if (text.isEmpty() || (length = text.length()) > 5) {
            return -1;
        }
        return length;
    }

    public boolean assignedAndChecked(String text) {
        String value;
        return call(text) || ((value = this.field) != null && value.isEmpty());
    }

    private boolean call(String text) {
        this.field = this.swapField;
        return text.isEmpty();
    }

    public boolean run(String text, String fieldValue) {
        this.field = null;
        this.swapField = fieldValue;
        return assignedAndChecked(text);
    }
}
