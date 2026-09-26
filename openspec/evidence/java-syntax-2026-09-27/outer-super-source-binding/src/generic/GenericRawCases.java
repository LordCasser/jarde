package generic;

@SuppressWarnings("rawtypes")
class GenericRawCases extends GenericParent {
    class Member {
        String choose(String value) {
            return GenericRawCases.super.choose(value);
        }
    }

    public static void main(String[] args) {
        GenericRawCases outer = new GenericRawCases();
        System.out.println(outer.new Member().choose("value"));
    }
}
