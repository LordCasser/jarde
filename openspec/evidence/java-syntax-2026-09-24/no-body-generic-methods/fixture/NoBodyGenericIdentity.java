package nobodygeneric;

import java.io.IOException;

public abstract class NoBodyGenericIdentity {
    public abstract <T extends Number> T echo(T value);

    public abstract <T extends Number> T checked(T value) throws IOException;
}
