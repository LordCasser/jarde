public final class Runner {
    public static void main(String[] args) {
        OrderedFieldConcat receiver = new OrderedFieldConcat(3, 5, 7);
        System.out.println("fields=" + OrderedFieldConcat.instanceFields(receiver));
        try {
            OrderedFieldConcat.instanceFields(null);
            System.out.println("nullReceiver=missing-npe");
        } catch (NullPointerException expected) {
            System.out.println("nullReceiver=NullPointerException events=" + receiver.events);
        }

        receiver = new OrderedFieldConcat(3, 5, 7);
        String plainPlus = OrderedFieldConcat.plainPlusObjectBeforeLaterField(receiver);
        System.out.println("plainPlusControl=" + plainPlus + " calls=" + receiver.toStringCalls
                + " events=" + receiver.events);

        receiver = new OrderedFieldConcat(3, 5, 7);
        String explicitValueOf = OrderedFieldConcat.explicitValueOfBeforeLaterField(receiver);
        System.out.println("explicitValueOfControl=" + explicitValueOf + " calls=" + receiver.toStringCalls
                + " events=" + receiver.events);

        receiver = new OrderedFieldConcat(3, 5, 7);
        receiver.payload = receiver;
        String builderField = OrderedFieldConcat.objectFieldBeforeLaterField(receiver);
        System.out.println("builderObjectFieldControl=" + builderField + " calls=" + receiver.toStringCalls
                + " events=" + receiver.events);

        receiver = new OrderedFieldConcat(3, 5, 7);
        String mutation = OrderedFieldConcat.objectBeforeLaterField(receiver);
        System.out.println("objectMutation=" + mutation + " calls=" + receiver.toStringCalls
                + " events=" + receiver.events);

        receiver = new OrderedFieldConcat(3, 5, 7);
        receiver.textResult = null;
        mutation = OrderedFieldConcat.objectBeforeLaterField(receiver);
        System.out.println("nullObjectToString=" + mutation + " calls=" + receiver.toStringCalls
                + " events=" + receiver.events);

        System.out.println("nullObject=" + OrderedFieldConcat.nullObject(null));
        char[] chars = new char[] {'x', 'y'};
        String charArray = OrderedFieldConcat.charArrayAsObject(chars);
        System.out.println("charArrayObject=" + (charArray.startsWith("object=[C@")
                && charArray.equals("object=" + String.valueOf((Object) chars))));

        receiver = new OrderedFieldConcat(3, 5, 7);
        receiver.payload = null;
        System.out.println("objectFieldNull=" + OrderedFieldConcat.objectFieldBeforeLaterField(receiver));
        receiver = new OrderedFieldConcat(3, 5, 7);
        receiver.payload = chars;
        String fieldCharArray = OrderedFieldConcat.objectFieldBeforeLaterField(receiver);
        System.out.println("objectFieldCharArray=" + (fieldCharArray.startsWith("object=[C@")
                && fieldCharArray.endsWith(";later=7")));
    }
}
