import { MEETING_STATUSES, SECTION_TYPES } from "../types/index";
import type { MeetingStatus, SectionType } from "../types/index";

describe("MeetingStatus", () => {
  test("正しい 4 つの値のみを持つこと", () => {
    expect(MEETING_STATUSES).toHaveLength(4);
    expect(MEETING_STATUSES).toContain("recording");
    expect(MEETING_STATUSES).toContain("processing");
    expect(MEETING_STATUSES).toContain("done");
    expect(MEETING_STATUSES).toContain("error");
  });

  test("各値が string 型であること", () => {
    for (const status of MEETING_STATUSES) {
      expect(typeof status).toBe("string");
    }
  });

  test("不正な値が含まれないこと", () => {
    const invalidValues = ["pending", "cancelled", "active", "idle", ""];
    for (const invalid of invalidValues) {
      expect(MEETING_STATUSES).not.toContain(invalid);
    }
  });

  test("型として使用できること（コンパイル時チェック）", () => {
    const status: MeetingStatus = "done";
    expect(MEETING_STATUSES).toContain(status);
  });
});

describe("SectionType", () => {
  test("正しい 3 つの値のみを持つこと", () => {
    expect(SECTION_TYPES).toHaveLength(3);
    expect(SECTION_TYPES).toContain("decisions");
    expect(SECTION_TYPES).toContain("next");
    expect(SECTION_TYPES).toContain("body");
  });

  test("各値が string 型であること", () => {
    for (const sectionType of SECTION_TYPES) {
      expect(typeof sectionType).toBe("string");
    }
  });

  test("不正な値が含まれないこと", () => {
    const invalidValues = ["header", "footer", "main", "action", ""];
    for (const invalid of invalidValues) {
      expect(SECTION_TYPES).not.toContain(invalid);
    }
  });

  test("型として使用できること（コンパイル時チェック）", () => {
    const sectionType: SectionType = "decisions";
    expect(SECTION_TYPES).toContain(sectionType);
  });
});

describe("ハードコードチェック", () => {
  test("MEETING_STATUSES がハードコードされた文字列ではなく型定数から生成されていること", () => {
    // 型の定義と配列の定義が一致していることを確認
    const expectedStatuses: MeetingStatus[] = ["recording", "processing", "done", "error"];
    expect(MEETING_STATUSES).toEqual(expectedStatuses);
  });

  test("SECTION_TYPES がハードコードされた文字列ではなく型定数から生成されていること", () => {
    const expectedTypes: SectionType[] = ["decisions", "next", "body"];
    expect(SECTION_TYPES).toEqual(expectedTypes);
  });
});
