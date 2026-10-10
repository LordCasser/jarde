import java.util.List;

public class VariablePostfixLoop {
	public static int countEmpty(List<String> list) {
		int i = 0;
		if (list != null) {
			for (String str : list) {
				if (str.isEmpty()) {
					i++;
				}
			}
		}
		return i;
	}
}
