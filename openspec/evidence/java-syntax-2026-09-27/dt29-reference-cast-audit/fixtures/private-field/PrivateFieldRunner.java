package dt29;

import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Method;

public class PrivateFieldRunner {
	public static void main(String[] args) throws Exception {
		Class<?> aType = Class.forName("dt29.PrivateFieldFamily$A");
		Class<?> bType = Class.forName("dt29.PrivateFieldFamily$B");
		Constructor<?> constructor = bType.getDeclaredConstructor();
		Object value = constructor.newInstance();
		Method set = bType.getMethod("set", boolean.class, boolean.class);
		Field visible = aType.getField("visible");
		Field hidden = aType.getDeclaredField("hidden");
		hidden.setAccessible(true);
		set.invoke(value, true, false);
		boolean actualVisible = visible.getBoolean(value);
		boolean actualHidden = hidden.getBoolean(value);
		if (!actualVisible || actualHidden) {
			throw new AssertionError(actualVisible + ":" + actualHidden);
		}
		System.out.println(actualVisible + ":" + actualHidden);
	}
}
