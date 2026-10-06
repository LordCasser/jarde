/// The interface entry the class's `Signature` spells with an argument, carrying a type-use
/// annotation on the `implements` clause. The class header cannot be published from the
/// `Signature` while that attribute is present, so the header stays the physical raw
/// `implements java.lang.Comparable` — and the erased contract then keeps its bridge visible.
public class TypeUse implements java.lang.@Mark Comparable<TypeUse> {
    public int compareTo(TypeUse other) {
        return 0;
    }

    public static void main(String[] args) {
        java.lang.Comparable<TypeUse> contract = new TypeUse();
        System.out.println(contract.compareTo(new TypeUse()));
    }
}
