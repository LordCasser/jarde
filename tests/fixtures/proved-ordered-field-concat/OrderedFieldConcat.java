public final class OrderedFieldConcat {
    public int value;
    public volatile int volatileValue;
    public int later;
    public Object payload;
    public int toStringCalls;
    public String textResult = "object";
    public String events = "";

    public OrderedFieldConcat(int value, int volatileValue, int later) {
        this.value = value;
        this.volatileValue = volatileValue;
        this.later = later;
    }

    private void event(String value) {
        if (!events.isEmpty()) {
            events += ",";
        }
        events += value;
    }

    public static String instanceFields(OrderedFieldConcat receiver) {
        return new StringBuilder().append("value=").append(receiver.value)
                .append(";volatile=").append(receiver.volatileValue).toString();
    }

    public static String plainPlusObjectBeforeLaterField(OrderedFieldConcat receiver) {
        return "object=" + receiver + ";later=" + receiver.later;
    }

    public static String explicitValueOfBeforeLaterField(OrderedFieldConcat receiver) {
        return "object=" + java.lang.String.valueOf((Object) receiver)
                + ";later=" + receiver.later;
    }

    public static String objectFieldBeforeLaterField(OrderedFieldConcat receiver) {
        return new StringBuilder().append("object=").append(receiver.payload)
                .append(";later=").append(receiver.later).toString();
    }

    public static String objectBeforeLaterField(OrderedFieldConcat receiver) {
        return new StringBuilder().append("object=").append((Object) receiver)
                .append(";later=").append(receiver.later).toString();
    }

    public static String nullObject(Object value) {
        return new StringBuilder().append("object=").append((Object) value).toString();
    }

    public static String charArrayAsObject(Object value) {
        return new StringBuilder().append("object=").append((Object) value).toString();
    }

    @Override
    public String toString() {
        toStringCalls++;
        later = 99;
        event("object.toString");
        return textResult;
    }
}
