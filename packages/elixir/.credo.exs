%{
  configs: [
    %{
      name: "default",
      files: %{
        included: ["lib/", "test/"],
        excluded: ["test/support"]
      },
      checks: :default
    }
  ]
}
