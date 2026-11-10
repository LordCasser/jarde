package flow;

import identity.Foo;
import java.util.List;

public class OwnedFieldCycle {
    public List<Foo> values;

    public void put(Base<List<Foo>> box) {
        values = box.getBase();
    }

    public List<Foo> get() {
        return values;
    }
}
