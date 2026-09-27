package em06;

/* JADX INFO: loaded from: fixture.jar:em06/FieldOrder.class */
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

    private void initBuilder(StringBuilder sb) {
        this.state = sb;
    }

    private int initField() {
        return this.state.length();
    }

    public String value() {
        return this.state.toString();
    }

    public static String statics() {
        return a + ":" + b + ":" + c + ":" + result;
    }
}
