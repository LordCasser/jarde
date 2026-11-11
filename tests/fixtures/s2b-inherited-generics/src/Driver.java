import java.util.ArrayList;
import java.util.List;

public class Driver {
    private static void require(boolean condition, String label) {
        if (!condition) {
            throw new AssertionError(label);
        }
    }

    public static void main(String[] args) {
        PlatformChild platform = new PlatformChild();
        require(platform.listRead() == null, "initial platform field");
        try {
            platform.returnInherited();
            throw new AssertionError("empty inherited read must throw");
        } catch (IndexOutOfBoundsException expected) {
            // Preserve the original ArrayList boundary behavior.
        }
        platform.add("alpha");
        platform.add("beta");
        require("alpha".equals(platform.returnInherited()), "inherited first element");
        require("beta".equals(platform.inheritedRead(1)), "inherited indexed element");
        try {
            platform.inheritedRead(-1);
            throw new AssertionError("negative inherited index must throw");
        } catch (IndexOutOfBoundsException expected) {
            // Both recovered calls must retain the real inherited implementation.
        }
        try {
            platform.inheritedRead(platform.size());
            throw new AssertionError("upper inherited index must throw");
        } catch (IndexOutOfBoundsException expected) {
            // Check both ends rather than only a successful lookup.
        }
        platform.add(null);
        require(platform.inheritedRead(2) == null, "null inherited element");
        List<String> values = new ArrayList<String>();
        values.add("gamma");
        platform.names = values;
        require(platform.listRead() == values, "platform field identity");
        platform.listRead().add("delta");
        require(platform.names.size() == 2, "platform field alias mutation");
        platform.names = null;
        require(platform.listRead() == null, "platform field null");

        CustomChild custom = new CustomChild();
        require(custom.getInherited() == null, "initial inherited getter");
        require(custom.readInheritedField() == null, "initial inherited field");
        custom.set(values);
        require(custom.getInherited() == values, "two-edge inherited getter identity");
        require(custom.readInheritedField() == values, "two-edge inherited field identity");
        require(custom.echo(values) == values, "generic echo identity");
        require(custom.echo(null) == null, "generic echo null");
        custom.readInheritedField().add("epsilon");
        require(custom.getInherited().size() == 3, "inherited alias mutation");
        List<String> replacement = new ArrayList<String>();
        custom.field = replacement;
        require(custom.getInherited() == replacement, "direct field replacement getter");
        require(custom.readInheritedField() == replacement, "direct field replacement reader");
        custom.set(null);
        require(custom.getInherited() == null, "inherited getter null");
        require(custom.readInheritedField() == null, "inherited field null");
        System.out.println("s2b-inherited-generics: platform and custom checks passed");
    }
}
