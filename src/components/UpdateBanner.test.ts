import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import UpdateBanner from "./UpdateBanner.vue";

const { check, relaunch } = vi.hoisted(() => ({
  check: vi.fn(),
  relaunch: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({ check }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch }));

describe("UpdateBanner", () => {
  beforeEach(() => {
    check.mockReset();
    relaunch.mockReset();
  });

  it("stays hidden when the automatic check finds no update", async () => {
    check.mockResolvedValue(null);
    const wrapper = mount(UpdateBanner);
    await flushPromises();
    expect(wrapper.find("aside").exists()).toBe(false);
  });

  it("reports a successful manual check when the app is current", async () => {
    check.mockResolvedValue(null);
    const wrapper = mount(UpdateBanner);
    await flushPromises();
    await (wrapper.vm as unknown as { checkForUpdate: (manual: boolean) => Promise<void> }).checkForUpdate(true);
    expect(wrapper.text()).toContain("Graft is up to date");
  });

  it("downloads an available update and relaunches", async () => {
    const downloadAndInstall = vi.fn().mockResolvedValue(undefined);
    check.mockResolvedValue({ version: "1.2.3", body: "Important fixes", downloadAndInstall });
    const wrapper = mount(UpdateBanner);
    await flushPromises();
    expect(wrapper.text()).toContain("Graft 1.2.3 is ready");
    await wrapper.get(".primary-button").trigger("click");
    await flushPromises();
    expect(downloadAndInstall).toHaveBeenCalledOnce();
    expect(relaunch).toHaveBeenCalledOnce();
  });
});
