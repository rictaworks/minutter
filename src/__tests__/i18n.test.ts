import { JA } from "../i18n/ja";

type NestedStringObject = {
  [key: string]: string | NestedStringObject;
};

function collectStrings(obj: NestedStringObject, path = ""): Array<{ path: string; value: string }> {
  const results: Array<{ path: string; value: string }> = [];
  for (const key of Object.keys(obj)) {
    const fullPath = path ? `${path}.${key}` : key;
    const value = obj[key];
    if (typeof value === "string") {
      results.push({ path: fullPath, value });
    } else if (typeof value === "object" && value !== null) {
      results.push(...collectStrings(value as NestedStringObject, fullPath));
    }
  }
  return results;
}

describe("JA i18n オブジェクト", () => {
  const allStrings = collectStrings(JA as unknown as NestedStringObject);

  test("すべての文字列キーが存在すること", () => {
    expect(allStrings.length).toBeGreaterThan(0);
  });

  test("すべての文字列値が空文字でないこと", () => {
    const emptyStrings = allStrings.filter(({ value }) => value.trim() === "");
    if (emptyStrings.length > 0) {
      const paths = emptyStrings.map(({ path }) => path).join(", ");
      throw new Error(`空の文字列が見つかりました: ${paths}`);
    }
    expect(emptyStrings).toHaveLength(0);
  });

  test("すべての値が string 型であること", () => {
    const nonStrings = allStrings.filter(({ value }) => typeof value !== "string");
    expect(nonStrings).toHaveLength(0);
  });

  test("appName が minutter であること", () => {
    expect(JA.appName).toBe("minutter");
  });

  test("modelError.downloadUrl が有効な URL であること", () => {
    expect(JA.modelError.downloadUrl).toMatch(/^https?:\/\//);
  });

  test("nav セクションに meetingList と newRecording が存在すること", () => {
    expect(JA.nav.meetingList).toBeTruthy();
    expect(JA.nav.newRecording).toBeTruthy();
  });

  test("status セクションに 4 つのステータスラベルが存在すること", () => {
    expect(JA.status.recording).toBeTruthy();
    expect(JA.status.processing).toBeTruthy();
    expect(JA.status.done).toBeTruthy();
    expect(JA.status.error).toBeTruthy();
  });
});
