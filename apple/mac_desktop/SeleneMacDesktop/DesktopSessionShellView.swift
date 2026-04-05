import SwiftUI

struct DesktopSessionShellView: View {
    var body: some View {
        HStack(alignment: .top, spacing: 20) {
            VStack(alignment: .leading, spacing: 16) {
                posturePanel

                sectionCard(
                    title: "History",
                    detail: "Bounded history placeholder aligned to the governed desktop session surface."
                )
            }
            .frame(width: 270, alignment: .topLeading)

            VStack(alignment: .leading, spacing: 16) {
                sectionCard(
                    title: "Session",
                    detail: "One dominant session surface placeholder for the cloud-authoritative Selene runtime."
                )
                .frame(maxWidth: .infinity, minHeight: 360, alignment: .topLeading)
            }
            .frame(maxWidth: .infinity, alignment: .topLeading)

            VStack(alignment: .leading, spacing: 16) {
                sectionCard(
                    title: "System Activity",
                    detail: "Read-only operational placeholder for governed sync, recovery, and alert posture."
                )

                sectionCard(
                    title: "Needs Attention",
                    detail: "Bounded actionable placeholder kept separate from transcript history."
                )
            }
            .frame(width: 300, alignment: .topLeading)
        }
        .padding(24)
        .frame(minWidth: 1180, minHeight: 720, alignment: .topLeading)
        .background(Color(nsColor: .windowBackgroundColor))
    }

    private var posturePanel: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                Text("Selene Mac Desktop")
                    .font(.largeTitle.weight(.bold))

                Text("First-class, non-authority")
                    .font(.headline)

                VStack(alignment: .leading, spacing: 8) {
                    posturePill("Wake word or explicit entry")
                    posturePill("Cloud authoritative")
                    posturePill("Session-bound placeholder")
                }

                Text("Bounded desktop placeholder surface only. No local authority, proof, governance, or law behavior.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
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
    DesktopSessionShellView()
}
