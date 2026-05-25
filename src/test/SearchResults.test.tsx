import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import SearchResults from "../components/SearchResults";
import type { ProfileSearchResults } from "../components/SearchResults";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("SearchResults", () => {
  it("shows empty state when there are no results", () => {
    render(
      <SearchResults
        profileResults={[]}
        query="foobar"
        onMovieSelect={vi.fn()}
        onShowSelect={vi.fn()}
      />
    );
    expect(screen.getByText(/No results for/)).toBeInTheDocument();
    expect(screen.getByText(/foobar/)).toBeInTheDocument();
  });

  it("renders the profile name and singular result count", () => {
    const profileResults: ProfileSearchResults[] = [
      { profile: "MyProfile", results: [{ kind: "movie", id: 1, name: "Inception", icon: "" }] },
    ];
    render(
      <SearchResults
        profileResults={profileResults}
        query="incep"
        onMovieSelect={vi.fn()}
        onShowSelect={vi.fn()}
      />
    );
    expect(screen.getByText(/MyProfile/)).toBeInTheDocument();
    expect(screen.getByText(/1 result(?!s)/)).toBeInTheDocument();
  });

  it("renders plural result count for multiple results", () => {
    const profileResults: ProfileSearchResults[] = [
      {
        profile: "P",
        results: [
          { kind: "movie", id: 1, name: "A", icon: "" },
          { kind: "movie", id: 2, name: "B", icon: "" },
        ],
      },
    ];
    render(
      <SearchResults
        profileResults={profileResults}
        query="x"
        onMovieSelect={vi.fn()}
        onShowSelect={vi.fn()}
      />
    );
    expect(screen.getByText(/2 results/)).toBeInTheDocument();
  });

  it("renders result card names", () => {
    const profileResults: ProfileSearchResults[] = [
      {
        profile: "MyProfile",
        results: [
          { kind: "movie", id: 1, name: "Inception", icon: "" },
          { kind: "show", id: 2, name: "Breaking Bad", icon: "" },
        ],
      },
    ];
    render(
      <SearchResults
        profileResults={profileResults}
        query="test"
        onMovieSelect={vi.fn()}
        onShowSelect={vi.fn()}
      />
    );
    expect(screen.getByText("Inception")).toBeInTheDocument();
    expect(screen.getByText("Breaking Bad")).toBeInTheDocument();
  });

  it("calls onMovieSelect with id and profile when a movie card is clicked", async () => {
    const onMovieSelect = vi.fn();
    const profileResults: ProfileSearchResults[] = [
      { profile: "MyProfile", results: [{ kind: "movie", id: 42, name: "Inception", icon: "" }] },
    ];
    render(
      <SearchResults
        profileResults={profileResults}
        query="incep"
        onMovieSelect={onMovieSelect}
        onShowSelect={vi.fn()}
      />
    );
    await userEvent.click(screen.getByText("Inception"));
    expect(onMovieSelect).toHaveBeenCalledWith(42, "MyProfile");
  });

  it("calls onShowSelect with id and profile when a show card is clicked", async () => {
    const onShowSelect = vi.fn();
    const profileResults: ProfileSearchResults[] = [
      { profile: "MyProfile", results: [{ kind: "show", id: 7, name: "Breaking Bad", icon: "" }] },
    ];
    render(
      <SearchResults
        profileResults={profileResults}
        query="break"
        onMovieSelect={vi.fn()}
        onShowSelect={onShowSelect}
      />
    );
    await userEvent.click(screen.getByText("Breaking Bad"));
    expect(onShowSelect).toHaveBeenCalledWith(7, "MyProfile");
  });
});
