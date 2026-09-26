package generic;

class GenericParent<T> {
    String choose(T value) {
        return "type-variable";
    }

    String choose(CharSequence value) {
        return "char-sequence";
    }
}

public class GenericCases extends GenericParent<String> {
    class Member {
        String choose(String value) {
            return GenericCases.super.choose(value);
        }
    }

    public static void main(String[] args) {
        GenericCases outer = new GenericCases();
        System.out.println(outer.new Member().choose("value"));
    }
}
