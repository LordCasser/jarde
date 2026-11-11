import java.util.List;

public class CustomChild extends Mid<String> {
    public List<String> getInherited() {
        return get();
    }

    public List<String> readInheritedField() {
        return field;
    }

    public List<String> echo(List<String> value) {
        return value;
    }
}
