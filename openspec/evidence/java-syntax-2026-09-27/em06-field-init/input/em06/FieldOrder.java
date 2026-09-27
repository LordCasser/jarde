package em06;

public class FieldOrder {
    private static final StringBuilder trace = new StringBuilder();
    private static final String a = trace.append("a").toString();
    private static final String b = trace.append("b").toString();
    private static final String c = trace.append("c").toString();
    private static final String result = trace.toString();

    private StringBuilder state;
    private int field;

    public FieldOrder() {
        initBuilder(new StringBuilder("sb"));
        this.field = initField();
        this.state.append(this.field);
    }

    private void initBuilder(StringBuilder value) {
        this.state = value;
    }

    private int initField() {
        return state.length();
    }

    public String value() {
        return state.toString();
    }

    public static String statics() {
        return a + ":" + b + ":" + c + ":" + result;
    }
}
