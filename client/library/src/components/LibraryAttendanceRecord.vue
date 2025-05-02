<template>
  <div class="print-container p-4">
    <div class="mb-4 flex justify-between items-center no-print">
      <div>
        <label for="date-selector" class="mr-2">Select Date:</label>
        <input
          type="date"
          id="date-selector"
          v-model="selectedDate"
          class="border p-1 rounded"
        />
      </div>
      <div>
        <button
          @click="downloadPDF"
          :disabled="isGeneratingPdf"
          class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded mr-2 disabled:opacity-50"
        >
          {{ isGeneratingPdf ? "Generating..." : "Download PDF" }}
        </button>
        <button
          @click="printReport"
          class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded no-print"
        >
          Print
        </button>
      </div>
    </div>

    <div id="pdf-render-area" class="pdf-render-area bg-white shadow-lg">
      <div class="page-content page-1">
        <div class="header common-header">
          <img
            src="/Gendejesus.png"
            alt="General de Jesus College Logo"
            class="header-logo"
          />
          <div class="header-text">
            <div class="college-name">General de Jesus College</div>
            <div class="address">
              VALLARTA ST. POBLACION, SAN ISIDRO, NUEVA ECIJA
            </div>
            <div class="contact">
              (+6344) 940-6161 | gdjcdejesus@gmail.com | gdjcdejesus.edu.ph
            </div>
          </div>
          <div class="report-title-line">
            Daily Record of Library Users SY: 2024-2025
            {{ formattedSelectedDateForTitle }}
          </div>
        </div>

        <div id="main-table-container">
          <table class="attendance-table">
            <thead>
              <tr>
                <th class="date-column column-name">DATE</th>
                <th class="time-column column-name">TIME</th>
                <th class="name-column column-name">
                  NAME (Last Name, First Name)
                </th>
                <th
                  v-for="course in courses"
                  :key="course"
                  class="course-column"
                >
                  <div class="vertical-text">{{ course }}</div>
                </th>
                <th class="purpose-column column-name">Purpose of Visit</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="attendanceData.length === 0">
                <td :colspan="5 + courses.length" class="text-center py-4">
                  No attendance data for selected date.
                </td>
              </tr>
              <tr
                v-for="(entry, index) in attendanceData"
                :key="`row-${index}`"
              >
                <td>{{ entry.date }}</td>
                <td>{{ entry.time }}</td>
                <td class="text-left pl-1">{{ entry.name }}</td>
                <td
                  v-for="course in courses"
                  :key="`cell-${index}-${course}`"
                  class="course-cell"
                >
                  {{ entry.course === course ? "✓" : "" }}
                </td>
                <td class="text-left pl-1">{{ entry.purpose }}</td>
              </tr>
              <tr class="total-row">
                <td colspan="3" class="text-right pr-2">
                  <strong>Total:</strong>
                </td>
                <td
                  v-for="course in courses"
                  :key="`total-${course}`"
                  class="total-cell"
                >
                  <strong>{{ getTotalForCourse(course) }}</strong>
                </td>
                <td class="text-right pr-2">
                  <strong>Grand Total: {{ attendanceData.length }}</strong>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
        <div class="footer print-only">
          Page <span class="page-number">1</span>
        </div>
      </div>

      <div class="page-content page-2 print-page-break">
        <div class="header common-header">
          <img
            src="/Gendejesus.png"
            alt="General de Jesus College Logo"
            class="header-logo"
          />
          <div class="header-text">
            <div class="college-name">General de Jesus College</div>
            <div class="address">
              VALLARTA ST. POBLACION, SAN ISIDRO, NUEVA ECIJA
            </div>
            <div class="contact">
              (+6344) 940-6161 | gdjcdejesus@gmail.com | gdjcdejesus.edu.ph
            </div>
          </div>
          <div class="report-title-line">
            Daily Record of Library Users SY: 2024-2025
            {{ formattedSelectedDateForTitle }}
          </div>
        </div>

        <div class="summary-section">
          <div class="summary-header full-width-header">
            Total Attendance by Purpose - {{ formattedSelectedDateForTitle }}
          </div>

          <div class="course-summary">
            <table class="course-summary-table">
              <tbody>
                <tr
                  v-for="(coursesChunk, chunkIndex) in chunkedCourses"
                  :key="`chunk-${chunkIndex}`"
                  class="course-summary-row"
                >
                  <template
                    v-for="(course, courseIndex) in coursesChunk"
                    :key="`summary-${course}`"
                  >
                    <td class="course-summary-label">
                      <strong>{{ course }}:</strong>
                    </td>
                    <td class="course-summary-value">
                      {{ getTotalForCourse(course) }}
                    </td>
                    <td
                      class="course-summary-spacer"
                      v-if="courseIndex < coursesChunk.length - 1"
                    ></td>
                  </template>
                  <template v-if="coursesChunk.length < 5">
                    <td
                      v-for="i in 5 - coursesChunk.length"
                      :key="`filler-${chunkIndex}-${i}`"
                      colspan="3"
                      class="course-summary-item empty"
                    ></td>
                  </template>
                </tr>
                <tr class="total-row course-total-row">
                  <td
                    colspan="15"
                    class="text-right pr-2 pt-2 border-t-2 border-black"
                  >
                    <strong
                      >Total Attendance: {{ attendanceData.length }}</strong
                    >
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="purpose-summary-container mt-4">
            <table class="purpose-summary-table">
              <thead>
                <tr>
                  <th
                    v-for="purpose in uniquePurposes"
                    :key="`purpose-header-${purpose}`"
                  >
                    {{ purpose.toUpperCase() }}
                  </th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td
                    v-for="purpose in uniquePurposes"
                    :key="`purpose-count-${purpose}`"
                  >
                    {{ getTotalForPurpose(purpose) }}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
        <div class="footer print-only">
          Page <span class="page-number">2</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, nextTick } from "vue";
import { useToast } from "@/components/ui/toast/use-toast";
import {
  usePdfGenerator,
  AttendanceEntry,
} from "@/composables/usePdfGenerator";

export default defineComponent({
  name: "LibraryAttendanceRecord",
  setup() {
    // Reactive state
    const selectedDate = ref("2025-02-11");
    const isGeneratingPdf = ref(false);
    const logoBase64 = ref<string | null>(null);
    const schoolYear = "2024-2025";

    // Sample data
    const attendanceData = ref<AttendanceEntry[]>([
      {
        date: "Feb. 11, 2025",
        time: "08:05 am",
        name: "DOE, JOHN R.",
        course: "BSIT",
        purpose: "Research",
      },
      {
        date: "Feb. 11, 2025",
        time: "08:15 am",
        name: "SMITH, JANE M.",
        course: "BEEd",
        purpose: "Borrowing Books",
      },
      {
        date: "Feb. 11, 2025",
        time: "08:30 am",
        name: "CRUZ, MARIA S.",
        course: "STEM",
        purpose: "Reading/Study",
      },
      {
        date: "Feb. 11, 2025",
        time: "08:45 am",
        name: "GARCIA, LUIS P.",
        course: "Faculty",
        purpose: "Consultation",
      },
      {
        date: "Feb. 11, 2025",
        time: "09:00 am",
        name: "REYES, ANNA L.",
        course: "ABM",
        purpose: "Group Work",
      },
      {
        date: "Feb. 11, 2025",
        time: "09:10 am",
        name: "SANTOS, PETER T.",
        course: "BSHM",
        purpose: "Internet Access",
      },
      {
        date: "Feb. 11, 2025",
        time: "09:20 am",
        name: "RAMOS, ELENA G.",
        course: "JHS",
        purpose: "Reading/Study",
      },
      {
        date: "Feb. 11, 2025",
        time: "11:00 am",
        name: "LEE, CHLOE K.",
        course: "GAS",
        purpose: "Research",
      },
      {
        date: "Feb. 11, 2025",
        time: "11:15 am",
        name: "WONG, DAVID Z.",
        course: "BSAIS",
        purpose: "Borrowing Books",
      },
      {
        date: "Feb. 11, 2025",
        time: "04:45 pm",
        name: "PEREZ, SOFIA B.",
        course: "BSEdEng",
        purpose: "Returning Books",
      },
    ]);

    const courses = [
      "JHS",
      "ABM",
      "STEM",
      "HUMMS",
      "GAS",
      "BEEd",
      "BSEdEng",
      "BSEdSoc",
      "BSA",
      "BSAIS",
      "BSMA",
      "BSIA",
      "BSBA",
      "BSBAFM",
      "BSBAHRDM",
      "BSBAMM",
      "BSIT",
      "BSHM",
      "Faculty",
    ];

    // Composable imports
    const { toast } = useToast();
    const { generatePDF } = usePdfGenerator();

    // Computed properties
    const formattedSelectedDateForTitle = computed(() => {
      const date = new Date(selectedDate.value + "T00:00:00");
      return date.toLocaleDateString("en-US", {
        month: "long",
        day: "numeric",
        year: "numeric",
      });
    });

    const chunkedCourses = computed(() => {
      const chunkSize = 5;
      const result = [];
      for (let i = 0; i < courses.length; i += chunkSize) {
        result.push(courses.slice(i, i + chunkSize));
      }
      return result;
    });

    const uniquePurposes = computed(() => {
      const purposes = new Set(
        attendanceData.value.map((entry) => entry.purpose)
      );
      return Array.from(purposes).sort();
    });

    // Helper functions
    const getTotalForCourse = (course: string) => {
      return attendanceData.value.filter((entry) => entry.course === course)
        .length;
    };

    const getTotalForPurpose = (purpose: string) => {
      return attendanceData.value.filter((entry) => entry.purpose === purpose)
        .length;
    };

    // Load logo function
    const loadLogo = async () => {
      try {
        const response = await fetch("/Gendejesus.png");
        if (!response.ok) throw new Error("Logo not found");
        const blob = await response.blob();

        return new Promise<string>((resolve, reject) => {
          const reader = new FileReader();
          reader.onloadend = () => {
            logoBase64.value = reader.result as string;
            resolve(logoBase64.value);
          };
          reader.onerror = reject;
          reader.readAsDataURL(blob);
        });
      } catch (error) {
        console.error("Error loading logo:", error);
        toast({
          title: "Logo Error",
          description: "Could not load the college logo.",
          variant: "destructive",
        });
        throw error;
      }
    };

    // PDF generation
    const downloadPDF = async () => {
      if (isGeneratingPdf.value) return;
      isGeneratingPdf.value = true;

      try {
        toast({ title: "Generating PDF", description: "Please wait..." });

        // Load logo if not already loaded
        if (!logoBase64.value) {
          await loadLogo();
        }

        if (!logoBase64.value) {
          toast({
            title: "Error",
            description: "Logo image not loaded. Cannot generate PDF.",
            variant: "destructive",
          });
          return;
        }

        const pdf = await generatePDF(
          attendanceData.value,
          courses,
          selectedDate.value,
          schoolYear,
          logoBase64.value,
          toast,
          formattedSelectedDateForTitle.value
        );

        pdf.save(`library-attendance-${selectedDate.value}.pdf`);

        toast({
          title: "PDF Generated Successfully",
          description: `library-attendance-${selectedDate.value}.pdf downloaded.`,
          variant: "success",
        });
      } catch (error) {
        console.error("Error generating PDF:", error);
        toast({
          title: "PDF Generation Failed",
          description:
            error instanceof Error
              ? error.message
              : "An unknown error occurred",
          variant: "destructive",
        });
      } finally {
        isGeneratingPdf.value = false;
      }
    };

    // Print function
    const printReport = async () => {
      toast({ title: "Preparing Print", description: "Please wait..." });
      await nextTick();
      window.print();
      toast({
        title: "Print Dialog Opened",
        description:
          "Adjust print settings as needed (Landscape, Scale: Fit to printable area).",
        variant: "success",
      });
    };

    return {
      selectedDate,
      formattedSelectedDateForTitle,
      courses,
      attendanceData,
      chunkedCourses,
      uniquePurposes,
      getTotalForCourse,
      getTotalForPurpose,
      downloadPDF,
      printReport,
      isGeneratingPdf,
    };
  },
});
</script>

<style scoped>
/* Base styles */
.print-container {
  width: 100%;
  max-width: 1300px; /* Approx landscape long bond paper width */
  margin: 0 auto;
  font-family: Arial, sans-serif; /* Match typical PDF fonts */
}

.pdf-render-area {
  border: 1px solid #ccc;
  margin-top: 1rem;
  padding: 0.5in; /* Simulate PDF margins for preview */
  /* Mimic paper size for preview - aspect ratio for 13x8.5 */
  /* width: 100%; */
  /* aspect-ratio: 13 / 8.5; */
  overflow: hidden; /* Hide anything overflowing the 'paper' */
}

/* Header Styles */
.common-header {
  display: flex;
  align-items: center; /* Vertically align logo and text */
  margin-bottom: 10px; /* Space below header */
  padding-bottom: 10px;
  /* border-bottom: 1px solid black; */ /* Optional separator */
}

.header-logo {
  width: 60px; /* Adjust size as needed */
  height: 60px;
  margin-right: 15px;
  object-fit: contain;
}

.header-text {
  flex-grow: 1;
  text-align: left;
}

.college-name {
  font-size: 16px; /* Slightly larger */
  font-weight: bold;
  font-family: "Times New Roman", Times, serif; /* Serif font like reference */
  margin-bottom: 2px;
}

.address,
.contact {
  font-size: 9px;
  line-height: 1.2;
}

.report-title-line {
  width: 100%;
  text-align: center;
  font-size: 11px;
  font-weight: bold;
  margin-top: 10px; /* Space above title line */
  position: absolute; /* Position relative to pdf-render-area */
  top: 85px; /* Adjust based on logo/header text height */
  left: 0;
}

/* Table Styles */
table {
  width: 100%;
  border-collapse: collapse;
  table-layout: fixed;
  margin-bottom: 15px;
  font-size: 8px; /* Base font size for table content */
}

th,
td {
  border: 1px solid black;
  padding: 1px 3px; /* Minimal padding */
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap; /* Prevent wrapping by default */
  vertical-align: middle; /* Center vertically */
}

th.column-name {
  font-weight: bold;
  font-size: 9px; /* Slightly larger header text */
  background-color: #e6e6e6; /* Light gray header */
  padding: 4px 3px;
}

.attendance-table .text-left {
  text-align: left;
}
.attendance-table .text-right {
  text-align: right;
}

.vertical-text {
  writing-mode: vertical-rl;
  text-orientation: mixed; /* Or upright */
  white-space: nowrap;
  transform: rotate(180deg);
  font-size: 8px; /* Small font for vertical */
  line-height: 1;
  padding: 5px 0px; /* Adjust padding */
  display: inline-block; /* Necessary for vertical alignment */
  margin: 0 auto; /* Center within the cell */
  max-height: 50px; /* Limit height */
}

/* Column Widths (adjust percentages based on content and total columns) */
.date-column {
  width: 7%;
}
.time-column {
  width: 6%;
}
.name-column {
  width: 18%;
}
.course-column {
  width: 3%;
  max-width: 25px;
  padding: 0;
} /* Fixed width for course cols */
.purpose-column {
  width: 15%;
}

.course-cell {
  font-size: 10px; /* Make checkmark slightly larger */
  font-weight: bold;
}

/* Total Row */
.total-row td {
  font-weight: bold;
  background-color: #f2f2f2;
  font-size: 8px;
  padding: 3px;
}
.total-cell {
  font-weight: bold;
  font-size: 9px;
}

/* Summary Section Styles */
.summary-section {
  margin-top: 20px; /* Space above summary */
  /* page-break-before: always; */ /* Force page break before this in print */
}

.full-width-header {
  width: 100%;
  background-color: #cccccc; /* Gray background */
  padding: 8px;
  text-align: center;
  color: black;
  font-size: 12px;
  font-weight: bold;
  margin: 15px 0;
  box-sizing: border-box;
}

/* Course Summary Table */
.course-summary {
  width: 100%;
  margin-bottom: 20px;
}
.course-summary-table {
  border-collapse: separate; /* Use separate for spacing */
  border-spacing: 0 2px; /* Add vertical space between rows */
  font-size: 9px;
  table-layout: auto; /* Allow columns to size based on content */
}
.course-summary-table td {
  border: none; /* No borders within the summary table cells */
  padding: 1px 5px;
  white-space: nowrap;
  text-align: left;
  vertical-align: top;
}

.course-summary-label {
  font-weight: bold;
  width: auto; /* Adjust as needed */
}
.course-summary-value {
  width: 30px; /* Fixed width for value */
  text-align: left;
}
.course-summary-spacer {
  width: 20px; /* Space between course pairs */
}

.course-total-row td {
  border-top: 2px solid black !important; /* Thick top border */
  padding-top: 5px !important;
  font-size: 10px;
}

/* Purpose Summary Table */
.purpose-summary-container {
  width: 70%; /* Make it less wide than the page */
  margin: 20px auto 0 auto; /* Center it */
}
.purpose-summary-table th,
.purpose-summary-table td {
  border: 1px solid black;
  padding: 4px;
  text-align: center;
  font-size: 9px;
}
.purpose-summary-table th {
  font-weight: bold;
  background-color: #e6e6e6;
}

/* Footer Styles */
.footer {
  text-align: right;
  font-size: 8px;
  margin-top: 15px;
  width: 100%;
  position: absolute; /* Position relative to page-content */
  bottom: 0.3in; /* Position near the bottom margin */
  right: 0.5in;
}

/* --- Print Specific Styles --- */
@media print {
  body {
    margin: 0;
    padding: 0;
    font-size: 8pt; /* Base font size for print */
    -webkit-print-color-adjust: exact !important; /* Force background colors (like headers) in Chrome/Safari */
    print-color-adjust: exact !important; /* Standard */
  }

  .no-print {
    display: none !important; /* Hide buttons, date selector */
  }

  .print-container {
    max-width: none; /* Remove max-width */
    margin: 0;
    padding: 0;
  }

  .pdf-render-area {
    border: none !important;
    box-shadow: none !important;
    margin: 0 !important;
    padding: 0 !important; /* Remove padding used for screen preview */
  }

  .page-content {
    page-break-inside: avoid; /* Try to keep content of a page together */
    padding: 0.5in; /* Add back margins for printing */
    position: relative; /* Needed for absolute footer positioning */
    height: 7.5in; /* Approx printable height on 8.5in landscape */
    width: 12in; /* Approx printable width on 13in landscape */
    box-sizing: border-box;
  }

  .print-page-break {
    page-break-before: always !important; /* Force break before summary */
  }

  .common-header {
    /* Headers should repeat via PDF generation, hide duplicate HTML headers in print */
    /* display: none; */ /* Or style them if needed */
  }

  /* Ensure table headers repeat on each page if table breaks (browser dependent) */
  thead {
    display: table-header-group;
  }
  /* Ensure total row stays with table */
  tbody tr.total-row {
    page-break-inside: avoid;
  }

  /* Summary section page break */
  .summary-section {
    margin-top: 0; /* Remove extra margin potentially added by page break */
  }

  .footer.print-only {
    display: block; /* Show the simple footer in print */
    position: fixed; /* Try fixed positioning for print footer */
    bottom: 10px;
    right: 20px;
  }

  /* Ensure vertical text prints correctly */
  .vertical-text {
    /* May need adjustments based on browser */
  }
}

/* Page Setup for Print */
@page {
  size: 13in 8.5in landscape; /* Long bond landscape */
  margin: 0.5in; /* Standard margins */
}
</style>
