import SwiftUI

struct SessionShellView: View {
    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                VStack(alignment: .leading, spacing: 10) {
                    Text("Selene iPhone")
                        .font(.largeTitle.weight(.bold))

                    Text("First-class, non-authority")
                        .font(.headline)

                    VStack(alignment: .leading, spacing: 8) {
                        posturePill("EXPLICIT_ONLY")
                        posturePill("Cloud authoritative")
                        posturePill("No wake parity claimed")
                    }

                    Text("Session-bound placeholder surface for the governed Selene runtime.")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                sectionCard(title: "Session", detail: "One dominant session surface placeholder.")

                sectionCard(title: "History", detail: "Bounded history placeholder.")

                sectionCard(title: "System Activity", detail: "Bounded governed activity placeholder.")
            }
            .padding(24)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .background(Color(.systemGroupedBackground))
    }

    private func sectionCard(title: String, detail: String) -> some View {
        GroupBox {
            Text(detail)
                .frame(maxWidth: .infinity, alignment: .leading)
        } label: {
            Text(title)
                .font(.headline)
        }
    }

    private func posturePill(_ text: String) -> some View {
        Text(text)
            .font(.caption.weight(.semibold))
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(Color.accentColor.opacity(0.12))
            .clipShape(Capsule())
    }
}

#Preview {
    SessionShellView()
}
