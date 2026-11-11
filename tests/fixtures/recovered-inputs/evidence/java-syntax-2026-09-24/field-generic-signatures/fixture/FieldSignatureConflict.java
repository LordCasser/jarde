package fieldsig;

import java.util.ArrayList;
import java.util.List;

public final class FieldSignatureConflict {
    public List<Object> items = new ArrayList<Object>();

    public void addInteger() {
        items.add(Integer.valueOf(42));
    }

    public Object first() {
        return items.get(0);
    }
}
