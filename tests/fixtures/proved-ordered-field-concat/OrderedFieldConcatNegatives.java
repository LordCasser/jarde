public final class OrderedFieldConcatNegatives {
    public static int staticValue = 17;

    public int value;
    public int later;
    public String events = "";

    public OrderedFieldConcatNegatives(int value, int later) {
        this.value = value;
        this.later = later;
    }

    private void event(String value) {
        if (!events.isEmpty()) {
            events += ",";
        }
        events += value;
    }

    public static String staticFieldBetweenAppends() {
        return new StringBuilder().append("value=").append(staticValue)
                .append(";done").toString();
    }

    public OrderedFieldConcatNegatives nextReceiver() {
        event("nextReceiver");
        return this;
    }

    public static String receiverCallShape(OrderedFieldConcatNegatives receiver) {
        return new StringBuilder().append("value=").append(receiver.nextReceiver().value)
                .append(";done").toString();
    }

    public void voidEffect() {
        event("voidEffect");
    }

    public static String voidEffectBetweenAppends(OrderedFieldConcatNegatives receiver) {
        StringBuilder builder = new StringBuilder().append("value=").append(receiver.value);
        receiver.voidEffect();
        return builder.append(";later=").append(receiver.later).toString();
    }

    public static String fieldHasExtraUse(OrderedFieldConcatNegatives receiver) {
        int observed = receiver.value;
        StringBuilder builder = new StringBuilder().append("value=").append(observed);
        return builder.append(";again=").append(observed).toString();
    }
}
