import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import Select from "../components/Select";

const OPTIONS = [
  { value: "en", label: "English" },
  { value: "fr", label: "French" },
  { value: "de", label: "German" },
];

describe("Select", () => {
  it("shows the placeholder when no value is selected", () => {
    render(<Select value="" onChange={() => {}} options={OPTIONS} placeholder="Pick a language" />);
    expect(screen.getByText("Pick a language")).toBeInTheDocument();
  });

  it("shows the label of the current value", () => {
    render(<Select value="fr" onChange={() => {}} options={OPTIONS} />);
    expect(screen.getByText("French")).toBeInTheDocument();
  });

  it("dropdown is hidden on initial render", () => {
    render(<Select value="" onChange={() => {}} options={OPTIONS} />);
    expect(screen.queryByText("English")).not.toBeInTheDocument();
  });

  it("opens the dropdown when clicked", async () => {
    render(<Select value="" onChange={() => {}} options={OPTIONS} />);
    await userEvent.click(screen.getByText("Select..."));
    expect(screen.getByText("English")).toBeInTheDocument();
    expect(screen.getByText("French")).toBeInTheDocument();
    expect(screen.getByText("German")).toBeInTheDocument();
  });

  it("calls onChange with the selected value and closes the dropdown", async () => {
    const handler = vi.fn();
    render(<Select value="" onChange={handler} options={OPTIONS} />);
    await userEvent.click(screen.getByText("Select..."));
    await userEvent.click(screen.getByText("German"));
    expect(handler).toHaveBeenCalledWith("de");
    expect(screen.queryByText("English")).not.toBeInTheDocument();
  });

  it("closes on outside click", async () => {
    render(
      <div>
        <Select value="" onChange={() => {}} options={OPTIONS} />
        <button>Outside</button>
      </div>
    );
    await userEvent.click(screen.getByText("Select..."));
    expect(screen.getByText("English")).toBeInTheDocument();
    await userEvent.click(screen.getByText("Outside"));
    expect(screen.queryByText("English")).not.toBeInTheDocument();
  });
});
