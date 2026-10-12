public final class NegativeRunner {
    public static void main(String[] args) {
        System.out.println("staticControl=" + OrderedFieldConcatNegatives.staticFieldBetweenAppends());
        OrderedFieldConcatNegatives receiver = new OrderedFieldConcatNegatives(3, 7);
        System.out.println("receiverCallControl=" + OrderedFieldConcatNegatives.receiverCallShape(receiver)
                + " events=" + receiver.events);
        receiver = new OrderedFieldConcatNegatives(3, 7);
        System.out.println("voidEffectControl=" + OrderedFieldConcatNegatives.voidEffectBetweenAppends(receiver)
                + " events=" + receiver.events);
        receiver = new OrderedFieldConcatNegatives(3, 7);
        System.out.println("extraUseControl=" + OrderedFieldConcatNegatives.fieldHasExtraUse(receiver));
    }
}
