public class UnknownArrayRelationRunner {
    public static void main(String[] args) {
        System.out.println(UnknownArrayRelation.childToBase(new UnknownArrayRelation.Child[0]));
        System.out.println(UnknownArrayRelation.childToMarker(new UnknownArrayRelation.MarkedChild[0]));
    }
}
