package dt29;

import java.lang.reflect.Field;

public class SamePackageParentRunner {
	public static void main(String[] args) throws Exception {
		Class<?> aType = Class.forName("dt29.SamePackageParentFamily$A");
		Class<?> bType = Class.forName("dt29.SamePackageParentFamily$B");
		Object value = bType.newInstance();
		bType.getMethod("set", boolean.class, boolean.class).invoke(value, true, false);

		Field parentProtected = aType.getDeclaredField("protectedField");
		Field parentPackage = aType.getDeclaredField("packagePrivateField");
		Field childProtected = bType.getDeclaredField("protectedField");
		Field childPackage = bType.getDeclaredField("packagePrivateField");
		Field parentPublic = aType.getDeclaredField("publicField");
		Field parentPrivate = aType.getDeclaredField("privateField");
		for (Field field : new Field[] { parentProtected, parentPackage, childProtected,
				childPackage, parentPublic, parentPrivate }) {
			field.setAccessible(true);
		}
		boolean[] values = {
				parentProtected.getBoolean(value), parentPackage.getBoolean(value),
				childProtected.getBoolean(value), childPackage.getBoolean(value)
		};
		if (!parentPublic.getBoolean(value) || parentPrivate.getBoolean(value)
				|| !values[0] || !values[1] || values[2] || values[3]) {
			throw new AssertionError("public/private control or owner binding changed");
		}
		System.out.println(values[0] + ":" + values[1] + ":" + values[2] + ":" + values[3]);
	}
}
